import 'dart:async';

import 'package:flutter/foundation.dart';

import '../models/bluetooth_device.dart';
import '../models/config.dart';
import '../models/telemetry.dart';
import '../models/trip.dart';
import 'bluetooth/bluetooth_coordinator.dart';
import 'bluetooth/bt_log_service.dart';
import 'bluetooth/incoming_message_dispatcher.dart';
import 'platform/native_bridge.dart';
import 'storage/storage_service.dart';
import 'trips/trip_tracker.dart';

/// Central state coordinator for the companion app.
///
/// All vehicle communication is transported over Bluetooth SPP.
class AppState extends ChangeNotifier {
  final StorageService storage;

  late final BtLogService logService;
  late final TripTracker tripTracker;
  late final BluetoothCoordinator btCoordinator;
  late final IncomingMessageDispatcher _dispatcher;

  CarTelemetry _telemetry = const CarTelemetry();
  PiConfig _config = const PiConfig();

  StreamSubscription? _btEventSub;
  StreamSubscription? _locEventSub;

  AppState(this.storage) {
    logService = BtLogService()..addListener(notifyListeners);
    tripTracker = TripTracker(storage)..addListener(notifyListeners);
    btCoordinator = BluetoothCoordinator(
      storage: storage,
      logService: logService,
    )..addListener(notifyListeners);

    _dispatcher = IncomingMessageDispatcher(
      onTpms: (tpms) {
        _telemetry = _telemetry.copyWith(tpms: tpms);
        notifyListeners();
      },
      onTelemetry: (telemetry) {
        _telemetry = telemetry;
        tripTracker.updateSpeed(_telemetry.speed);
        notifyListeners();
      },
      onConfig: (config) {
        _config = config;
        storage.setFlId(config.flSensorId);
        storage.setFrId(config.frSensorId);
        storage.setRlId(config.rlSensorId);
        storage.setRrId(config.rrSensorId);
        storage.setAlertPressure(config.pressureAlert);
        storage.setAlertTemp(config.highTempAlert);
        notifyListeners();
      },
      onBtStatus: btCoordinator.handleBtStatusMessage,
      onBtDeviceFound: btCoordinator.handleDeviceFound,
      onBtPairResult: (message) {
        final mac = message['mac'] as String? ?? '';
        final success = message['success'] as bool? ?? false;
        final detail = message['message'] as String? ?? '';
        logService.addLog(
          '[BT-PAIR] $mac: ${success ? "success" : "failed ($detail)"}',
        );
        btCoordinator.loadPairedDevices();
      },
      onBtLogs: logService.addLogs,
      onBtLogEntry: (line) => logService.addLog('[HUB-LOG] $line'),
      onBtPinCode: (pin, mac, name) =>
          btCoordinator.handlePinRequest(pin, mac, name),
      onBtDiscoveryState: btCoordinator.setDiscoveryState,
      onPiDiscovered: (mac, name) =>
          btCoordinator.handlePiDiscovered(mac, name),
      onLog: logService.addLog,
    );

    _init();
  }

  bool get isBtConnected => btCoordinator.isConnected;
  bool get isBtConnecting => btCoordinator.isConnecting;
  String get connectedMac => btCoordinator.connectedMac;
  String get connectedName => btCoordinator.connectedName;
  bool get isDiscovering => btCoordinator.isDiscovering;
  List<BtDeviceInfo> get discoveredDevices => btCoordinator.discoveredDevices;
  List<BtDeviceInfo> get pairedDevices => btCoordinator.pairedDevices;
  String? get pendingPinCode => btCoordinator.pendingPinCode;
  String? get pendingPinDeviceMac => btCoordinator.pendingPinDeviceMac;
  String? get pendingPinDeviceName => btCoordinator.pendingPinDeviceName;
  CarTelemetry get telemetry => _telemetry;
  PiConfig get config => _config;
  bool get isTpmsEnabled => _config.tpmsEnabled;
  List<String> get btLogs => logService.logs;
  List<TripRecord> get trips => tripTracker.trips;
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

    _btEventSub = NativeBridge.btEvents.listen(_handleBtEvent);
    _locEventSub = NativeBridge.locationEvents.listen(_handleLocationEvent);

    if (storage.gpsForeground) {
      await NativeBridge.startForegroundService();
      await NativeBridge.startGps(rateHz: 2);
    }

    await btCoordinator.loadPairedDevices();
    if (storage.autoReconnect && storage.piMac.isNotEmpty) {
      btCoordinator.connectToPi(storage.piMac);
    }
  }

  void _handleBtEvent(Map<String, dynamic> event) {
    switch (event['type'] as String?) {
      case 'connection':
        btCoordinator.handleBtConnectionEvent(
          event['connected'] as bool? ?? false,
          event['mac'] as String? ?? '',
          event['name'] as String? ?? '',
        );
        break;
      case 'message':
        _dispatcher.process(event['message'] as String? ?? '');
        break;
      case 'deviceFound':
        btCoordinator.handleDeviceFound(event);
        break;
      case 'discoveryState':
        btCoordinator.setDiscoveryState(event['active'] as bool? ?? false);
        break;
    }
  }

  void _handleLocationEvent(Map<String, dynamic> location) {
    final speedMs = (location['speed'] as num?)?.toDouble() ?? 0.0;
    final speedKmh = (speedMs * 3.6).clamp(0.0, 300.0);

    if (_telemetry.speed == 0.0 || !isBtConnected) {
      _telemetry = _telemetry.copyWith(speed: speedKmh);
      tripTracker.updateSpeed(speedKmh);
      notifyListeners();
    }

    if (isBtConnected) {
      sendBtCommand({
        'type': 'telemetry',
        'spd': speedKmh,
        'lat': (location['latitude'] as num?)?.toDouble() ?? 0.0,
        'lon': (location['longitude'] as num?)?.toDouble() ?? 0.0,
        'alt': (location['altitude'] as num?)?.toDouble() ?? 0.0,
        'heading': (location['bearing'] as num?)?.toDouble() ?? 0.0,
      });
    }
  }

  Future<void> matchWithPi() => btCoordinator.matchWithPi();
  Future<void> loadPairedDevices() => btCoordinator.loadPairedDevices();
  Future<void> startDiscovery() => btCoordinator.startDiscovery();
  Future<void> stopDiscovery() => btCoordinator.stopDiscovery();
  Future<void> pairDevice(String mac) => btCoordinator.pairDevice(mac);
  Future<void> unpairDevice(String mac) => btCoordinator.unpairDevice(mac);
  Future<void> connectToPi(String mac) => btCoordinator.connectToPi(mac);
  void disconnectPi() => btCoordinator.disconnectPi();
  Future<bool> sendBtCommand(Map<String, dynamic> command) => btCoordinator.sendBtCommand(command);
  void syncTpmsFromPi() => btCoordinator.syncTpms();
  void dismissPinDialog() => btCoordinator.dismissPinDialog();
  void clearLogs() => logService.clear();

  Future<void> saveConfig(PiConfig newConfig) async {
    _config = newConfig;
    await storage.setFlId(newConfig.flSensorId);
    await storage.setFrId(newConfig.frSensorId);
    await storage.setRlId(newConfig.rlSensorId);
    await storage.setRrId(newConfig.rrSensorId);
    await storage.setAlertPressure(newConfig.pressureAlert);
    await storage.setAlertTemp(newConfig.highTempAlert);
    await storage.setTpmsEnabled(newConfig.tpmsEnabled);

    if (isBtConnected) {
      sendBtCommand({
        'type': 'set_config',
        'payload': {
          'action': 'update_config',
          'fl': newConfig.flSensorId,
          'fr': newConfig.frSensorId,
          'rl': newConfig.rlSensorId,
          'rr': newConfig.rrSensorId,
          'alert': newConfig.pressureAlert,
          'high_temp_alert': newConfig.highTempAlert,
          'gps_enabled': newConfig.gpsEnabled,
          'tpms_enabled': newConfig.tpmsEnabled,
          'enabled': newConfig.tpmsEnabled,
        },
      });
    }
    notifyListeners();
  }

  Future<void> setTpmsEnabled(bool enabled) => saveConfig(_config.copyWith(tpmsEnabled: enabled));
  void finishActiveTrip() => tripTracker.finishActiveTrip();
  void clearTrips() => tripTracker.clearTrips();

  @override
  void dispose() {
    _btEventSub?.cancel();
    _locEventSub?.cancel();
    btCoordinator.dispose();
    tripTracker.dispose();
    super.dispose();
  }
}
