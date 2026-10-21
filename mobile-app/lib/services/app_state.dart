import 'dart:async';
import 'package:flutter/foundation.dart';
import '../models/telemetry.dart';
import '../models/config.dart';
import '../models/trip.dart';
import '../models/bluetooth_device.dart';
import 'storage/storage_service.dart';
import 'platform/native_bridge.dart';
import 'network/wifi_ws_service.dart';
import 'network/incoming_message_dispatcher.dart';
import 'bluetooth/bt_log_service.dart';
import 'bluetooth/bluetooth_coordinator.dart';
import 'trips/trip_tracker.dart';

/// Central state coordinator for the companion app.
///
/// Composes modular sub-services for storage, networking, Bluetooth coordination,
/// trip tracking, and logging, exposing a unified reactive interface for UI widgets.
class AppState extends ChangeNotifier {
  /// Local persistence service for settings and historical trips.
  final StorageService storage;

  /// Fallback WebSocket service for Wi-Fi hotspot communication.
  final WifiWsService wifiWs = WifiWsService();

  /// Logging service capturing Bluetooth daemon and connection events.
  late final BtLogService logService;

  /// Tracking service for calculating trip distance, speed, and idle timeouts.
  late final TripTracker tripTracker;

  /// Coordinator managing Bluetooth SPP connection lifecycle and device discovery.
  late final BluetoothCoordinator btCoordinator;

  late final IncomingMessageDispatcher _dispatcher;

  bool _wifiNetworkAvailable = false;
  bool _wifiSocketConnected = false;
  CarTelemetry _telemetry = const CarTelemetry();
  PiConfig _config = const PiConfig();

  StreamSubscription? _btEventSub;
  StreamSubscription? _locEventSub;
  StreamSubscription? _wifiEventSub;
  StreamSubscription? _wsMsgSub;
  StreamSubscription? _wsConnSub;

  /// Creates an [AppState] coordinator backed by the provided [storage].
  AppState(this.storage) {
    logService = BtLogService()..addListener(notifyListeners);
    tripTracker = TripTracker(storage)..addListener(notifyListeners);
    btCoordinator = BluetoothCoordinator(
      storage: storage,
      wifiWs: wifiWs,
      logService: logService,
    )..addListener(notifyListeners);

    _dispatcher = IncomingMessageDispatcher(
      onTpms: (tpms) {
        _telemetry = _telemetry.copyWith(tpms: tpms);
        notifyListeners();
      },
      onTelemetry: (t) {
        _telemetry = t;
        tripTracker.updateSpeed(_telemetry.speed);
        notifyListeners();
      },
      onConfig: (cfg) {
        _config = cfg;
        storage.setFlId(cfg.flSensorId);
        storage.setFrId(cfg.frSensorId);
        storage.setRlId(cfg.rlSensorId);
        storage.setRrId(cfg.rrSensorId);
        storage.setAlertPressure(cfg.pressureAlert);
        storage.setAlertTemp(cfg.highTempAlert);
        notifyListeners();
      },
      onBtStatus: btCoordinator.handleBtStatusMessage,
      onBtDeviceFound: btCoordinator.handleDeviceFound,
      onBtPairResult: (msg) {
        final mac = msg['mac'] as String? ?? '';
        final success = msg['success'] as bool? ?? false;
        final message = msg['message'] as String? ?? '';
        logService.addLog('[BT-PAIR] Ghép nối với $mac: ${success ? "Thành công" : "Thất bại ($message)"}');
        btCoordinator.loadPairedDevices();
      },
      onBtLogs: logService.addLogs,
      onBtLogEntry: (line) => logService.addLog('[HUB-LOG] $line'),
      onLog: logService.addLog,
    );

    _init();
  }

  // --- Getters ---

  /// Indicates whether the RFCOMM Bluetooth SPP connection is actively open.
  bool get isBtConnected => btCoordinator.isConnected;

  /// Indicates whether a Bluetooth connection attempt is currently in progress.
  bool get isBtConnecting => btCoordinator.isConnecting;

  /// The MAC address of the connected vehicle Bluetooth peer.
  String get connectedMac => btCoordinator.connectedMac;

  /// The friendly name of the connected vehicle Bluetooth peer.
  String get connectedName => btCoordinator.connectedName;

  /// Indicates whether the local Wi-Fi hotspot connection is active.
  bool get isWifiConnected => _wifiNetworkAvailable || _wifiSocketConnected;

  /// Indicates whether Bluetooth device discovery scanning is currently running.
  bool get isDiscovering => btCoordinator.isDiscovering;

  /// List of newly discovered nearby Bluetooth devices.
  List<BtDeviceInfo> get discoveredDevices => btCoordinator.discoveredDevices;

  /// List of devices currently paired and bonded with the vehicle hub.
  List<BtDeviceInfo> get pairedDevices => btCoordinator.pairedDevices;

  /// The generated PIN code awaiting user confirmation, or `null`.
  String? get pendingPinCode => btCoordinator.pendingPinCode;

  /// The MAC address of the device requesting PIN confirmation, or `null`.
  String? get pendingPinDeviceMac => btCoordinator.pendingPinDeviceMac;

  /// The name of the device requesting PIN confirmation, or `null`.
  String? get pendingPinDeviceName => btCoordinator.pendingPinDeviceName;

  /// The current vehicle telemetry including speed, RPM, and TPMS data.
  CarTelemetry get telemetry => _telemetry;

  /// The active sensor ID and threshold alert configuration.
  PiConfig get config => _config;

  /// Whether the Tire Pressure Monitoring System is currently active.
  bool get isTpmsEnabled => _config.tpmsEnabled;

  /// An unmodifiable view of all recorded Bluetooth daemon logs.
  List<String> get btLogs => logService.logs;

  /// An unmodifiable view of all completed trip journeys.
  List<TripRecord> get trips => tripTracker.trips;

  /// The currently active in-progress trip, or `null` if the vehicle is idle.
  TripRecord? get activeTrip => tripTracker.activeTrip;

  void _init() async {
    _config = PiConfig(
      flSensorId: storage.flId,
      frSensorId: storage.frId,
      rlSensorId: storage.rlId,
      rrSensorId: storage.rrId,
      pressureAlert: storage.alertPressure,
      highTempAlert: storage.alertTemp,
      gpsEnabled: storage.gpsForeground,
      tpmsEnabled: storage.tpmsEnabled,
    );

    // Setup native bridge event listeners
    _btEventSub = NativeBridge.btEvents.listen(_handleBtEvent);
    _locEventSub = NativeBridge.locationEvents.listen(_handleLocationEvent);
    _wifiEventSub = NativeBridge.wifiEvents.listen((e) {
      _wifiNetworkAvailable = e['connected'] as bool? ?? false;
      notifyListeners();
    });

    // Setup Wi-Fi WS listeners
    _wsMsgSub = wifiWs.messages.listen(_handleWsMessage);
    _wsConnSub = wifiWs.connectionState.listen((connected) {
      _wifiSocketConnected = connected;
      if (connected) {
        wifiWs.send({'type': 'get_bt_status'});
        wifiWs.send({'type': 'match_pi'});
      }
      notifyListeners();
    });

    if (storage.gpsForeground) {
      await NativeBridge.startForegroundService();
      await NativeBridge.startGps(rateHz: 2);
    }

    await btCoordinator.loadPairedDevices();

    if (storage.autoReconnect && storage.piMac.isNotEmpty) {
      btCoordinator.connectToPi(storage.piMac);
    }

    // Android may keep using mobile data when the selected car hotspot has no
    // internet. Bind to that Wi-Fi network before opening the WebSocket, and
    // keep the request active so a later Wi-Fi switch is picked up as well.
    await NativeBridge.bindToCarWifi();
    await wifiWs.connect();
  }

  void _handleBtEvent(Map<String, dynamic> event) {
    final type = event['type'] as String?;
    switch (type) {
      case 'connection':
        final connected = event['connected'] as bool? ?? false;
        final mac = event['mac'] as String? ?? '';
        final name = event['name'] as String? ?? '';
        btCoordinator.handleBtConnectionEvent(connected, mac, name);
        break;

      case 'message':
        final raw = event['message'] as String? ?? '';
        _dispatcher.process(raw, 'BT');
        break;
    }
  }

  void _handleLocationEvent(Map<String, dynamic> loc) {
    final speedMs = (loc['speed'] as num?)?.toDouble() ?? 0.0;
    final speedKmh = (speedMs * 3.6).clamp(0.0, 300.0);

    if (_telemetry.speed == 0.0 || !isBtConnected) {
      _telemetry = _telemetry.copyWith(speed: speedKmh);
      tripTracker.updateSpeed(speedKmh);
      notifyListeners();
    }
  }

  void _handleWsMessage(Map<String, dynamic> msg) {
    final type = msg['type'] as String?;
    switch (type) {
      case 'welcome':
      case 'pi_info':
      case 'pi_matched':
        final piMac = (msg['pi_mac'] as String? ?? '').trim().toUpperCase();
        if (piMac.isNotEmpty) {
          final piName = (msg['pi_name'] as String? ?? 'ICarMeter').trim();
          btCoordinator.handlePiDiscovered(piMac, piName.isNotEmpty ? piName : 'ICarMeter');
        }
        break;
      case 'bt_pin_code':
        btCoordinator.handlePinRequest(
          msg['pin'] as String? ?? '',
          msg['mac'] as String? ?? '',
          msg['name'] as String? ?? '',
        );
        break;
      case 'bt_status':
        final data = (msg['data'] as Map<String, dynamic>?) ?? msg;
        final piMac = ((data['pi_mac'] ?? msg['pi_mac']) as String? ?? '').trim().toUpperCase();
        if (piMac.isNotEmpty) {
          final piName = ((data['pi_name'] ?? msg['pi_name']) as String? ?? 'ICarMeter').trim();
          btCoordinator.handlePiDiscovered(piMac, piName.isNotEmpty ? piName : 'ICarMeter');
        }
        btCoordinator.handleBtStatusMessage(msg);
        break;
      case 'bt_device_found':
        btCoordinator.handleDeviceFound(msg);
        break;
      case 'bt_discovery_started':
        btCoordinator.setDiscoveryState(true);
        break;
      case 'bt_discovery_stopped':
        btCoordinator.setDiscoveryState(false);
        break;
      case 'bt_pair_result':
        final mac = msg['mac'] as String? ?? '';
        final success = msg['success'] as bool? ?? false;
        final message = msg['message'] as String? ?? '';
        logService.addLog('[BT-PAIR] Ghép nối với $mac: ${success ? "Thành công" : "Thất bại ($message)"}');
        btCoordinator.loadPairedDevices();
        break;
      case 'bt_logs':
        if (msg['logs'] is List) {
          logService.addLogs(List<String>.from(msg['logs'] as List));
        }
        break;
      case 'bt_log_entry':
        if (msg['line'] != null) {
          logService.addLog('[HUB-LOG] ${msg['line']}');
        }
        break;
      default:
        if (msg.containsKey('paired_devices')) {
          btCoordinator.handleBtStatusMessage(msg);
        }
        break;
    }
  }

  /// Sends a matching command to identify and pair with the Raspberry Pi.
  Future<void> matchWithPi() => btCoordinator.matchWithPi();

  /// Requests the hub daemon to return the latest list of paired Bluetooth devices.
  Future<void> loadPairedDevices() => btCoordinator.loadPairedDevices();

  /// Starts scanning for nearby Bluetooth devices.
  Future<void> startDiscovery() => btCoordinator.startDiscovery();

  /// Stops scanning for nearby Bluetooth devices.
  Future<void> stopDiscovery() => btCoordinator.stopDiscovery();

  /// Initiates a Bluetooth pairing request with the target [mac].
  Future<void> pairDevice(String mac) => btCoordinator.pairDevice(mac);

  /// Unpairs and removes the bond with the target [mac].
  Future<void> unpairDevice(String mac) => btCoordinator.unpairDevice(mac);

  /// Connects to the Raspberry Pi Bluetooth SPP service using [mac].
  Future<void> connectToPi(String mac) => btCoordinator.connectToPi(mac);

  /// Disconnects the active Bluetooth SPP connection with the vehicle.
  void disconnectPi() => btCoordinator.disconnectPi();

  /// Transmits a JSON [cmd] across the active Bluetooth SPP socket.
  bool sendBtCommand(Map<String, dynamic> cmd) => btCoordinator.sendBtCommand(cmd);

  /// Requests fresh TPMS configuration and telemetry readings over Bluetooth.
  void syncTpmsFromPi() => btCoordinator.syncTpms();

  /// Dismisses the pairing PIN confirmation overlay dialog.
  void dismissPinDialog() => btCoordinator.dismissPinDialog();

  /// Empties the in-memory Bluetooth daemon log buffer.
  void clearLogs() => logService.clear();

  // --- Config Actions ---

  /// Persists [newCfg] locally and transmits the update over Bluetooth SPP.
  Future<void> saveConfig(PiConfig newCfg) async {
    _config = newCfg;
    await storage.setFlId(newCfg.flSensorId);
    await storage.setFrId(newCfg.frSensorId);
    await storage.setRlId(newCfg.rlSensorId);
    await storage.setRrId(newCfg.rrSensorId);
    await storage.setAlertPressure(newCfg.pressureAlert);
    await storage.setAlertTemp(newCfg.highTempAlert);
    await storage.setTpmsEnabled(newCfg.tpmsEnabled);

    if (isBtConnected) {
      sendBtCommand({
        'type': 'set_config',
        'payload': {
          'action': 'update_config',
          'fl': newCfg.flSensorId,
          'fr': newCfg.frSensorId,
          'rl': newCfg.rlSensorId,
          'rr': newCfg.rrSensorId,
          'alert': newCfg.pressureAlert,
          'high_temp_alert': newCfg.highTempAlert,
          'gps_enabled': newCfg.gpsEnabled,
          'tpms_enabled': newCfg.tpmsEnabled,
          'enabled': newCfg.tpmsEnabled,
        }
      });
    }
    notifyListeners();
  }

  /// Toggles the TPMS subsystem on or off, saves the state, and transmits to hardware.
  Future<void> setTpmsEnabled(bool enabled) async {
    final updated = _config.copyWith(tpmsEnabled: enabled);
    await saveConfig(updated);
  }

  // --- Trip Actions ---

  /// Finishes the active trip, writes it to storage, and resets active tracking.
  void finishActiveTrip() => tripTracker.finishActiveTrip();

  /// Clears all stored trips and resets historical records.
  void clearTrips() => tripTracker.clearTrips();

  /// Releases subscriptions, timers, and sub-services.
  @override
  void dispose() {
    _btEventSub?.cancel();
    _locEventSub?.cancel();
    _wifiEventSub?.cancel();
    _wsMsgSub?.cancel();
    _wsConnSub?.cancel();
    btCoordinator.dispose();
    tripTracker.dispose();
    wifiWs.dispose();
    super.dispose();
  }
}
