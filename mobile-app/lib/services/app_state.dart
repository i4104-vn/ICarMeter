import 'dart:async';
import 'dart:convert';
import 'package:flutter/foundation.dart';
import '../models/telemetry.dart';
import '../models/config.dart';
import '../models/trip.dart';
import '../models/bluetooth_device.dart';
import 'storage_service.dart';
import 'native_bridge.dart';
import 'wifi_ws_service.dart';

class AppState extends ChangeNotifier {
  final StorageService storage;
  final WifiWsService wifiWs = WifiWsService();

  // Connection
  bool _isBtConnected = false;
  bool _isBtConnecting = false;
  String _connectedMac = '';
  String _connectedName = '';
  bool _isWifiConnected = false;

  // Bluetooth Scanning & Pairing
  bool _isDiscovering = false;
  final Map<String, BtDeviceInfo> _discoveredDevices = {};
  List<BtDeviceInfo> _pairedDevices = [];

  // Pairing PIN
  String? _pendingPinCode;
  String? _pendingPinDeviceMac;
  String? _pendingPinDeviceName;

  // Data
  CarTelemetry _telemetry = const CarTelemetry();
  PiConfig _config = const PiConfig();
  final List<String> _btLogs = [];

  // Trips
  List<TripRecord> _trips = [];
  TripRecord? _activeTrip;
  Timer? _tripIdleTimer;

  // Auto reconnect
  Timer? _reconnectTimer;
  StreamSubscription? _btEventSub;
  StreamSubscription? _locEventSub;
  StreamSubscription? _wifiEventSub;
  StreamSubscription? _wsMsgSub;
  StreamSubscription? _wsConnSub;

  AppState(this.storage) {
    _init();
  }

  bool get isBtConnected => _isBtConnected;
  bool get isBtConnecting => _isBtConnecting;
  String get connectedMac => _connectedMac;
  String get connectedName => _connectedName;
  bool get isWifiConnected => _isWifiConnected;

  bool get isDiscovering => _isDiscovering;
  List<BtDeviceInfo> get discoveredDevices => _discoveredDevices.values.toList();
  List<BtDeviceInfo> get pairedDevices => _pairedDevices;

  String? get pendingPinCode => _pendingPinCode;
  String? get pendingPinDeviceMac => _pendingPinDeviceMac;
  String? get pendingPinDeviceName => _pendingPinDeviceName;

  CarTelemetry get telemetry => _telemetry;
  PiConfig get config => _config;
  List<String> get btLogs => List.unmodifiable(_btLogs);

  List<TripRecord> get trips => List.unmodifiable(_trips);
  TripRecord? get activeTrip => _activeTrip;

  void _init() async {
    _trips = storage.getTrips();
    _config = PiConfig(
      flSensorId: storage.flId,
      frSensorId: storage.frId,
      rlSensorId: storage.rlId,
      rrSensorId: storage.rrId,
      pressureAlert: storage.alertPressure,
      highTempAlert: storage.alertTemp,
      gpsEnabled: storage.gpsForeground,
    );

    // Setup native event listeners
    _btEventSub = NativeBridge.btEvents.listen(_handleBtEvent);
    _locEventSub = NativeBridge.locationEvents.listen(_handleLocationEvent);
    _wifiEventSub = NativeBridge.wifiEvents.listen(_handleWifiState);

    // Setup Wi-Fi WS listeners
    _wsMsgSub = wifiWs.messages.listen(_handleWsMessage);
    _wsConnSub = wifiWs.connectionState.listen((connected) {
      _isWifiConnected = connected;
      notifyListeners();
    });

    // Start background services if enabled
    if (storage.gpsForeground) {
      await NativeBridge.startForegroundService();
      await NativeBridge.startGps(rateHz: 2);
    }

    await loadPairedDevices();

    // Auto connect to saved device MAC if present
    if (storage.autoReconnect && storage.piMac.isNotEmpty) {
      connectToPi(storage.piMac);
    }

    // Connect to Wi-Fi fallback WebSocket
    wifiWs.connect();
  }

  void _handleBtEvent(Map<String, dynamic> event) {
    final type = event['type'] as String?;
    switch (type) {
      case 'connection':
        final connected = event['connected'] as bool? ?? false;
        _isBtConnected = connected;
        _isBtConnecting = false;
        if (connected) {
          _connectedMac = event['mac'] as String? ?? '';
          _connectedName = event['name'] as String? ?? 'Thiết bị trên xe';
          _addLog('[BT-SPP] Đã kết nối với $_connectedName ($_connectedMac)');
          // Request configuration over Bluetooth SPP
          sendBtCommand({'type': 'get_config'});
          sendBtCommand({'type': 'get_bt_logs'});
        } else {
          _addLog('[BT-SPP] Đã ngắt kết nối Bluetooth SPP');
          _connectedMac = '';
          _connectedName = '';
          _scheduleAutoReconnect();
        }
        notifyListeners();
        break;

      case 'message':
        final raw = event['message'] as String? ?? '';
        _processIncomingMessage(raw, 'BT');
        break;

    }
  }

  void _handleLocationEvent(Map<String, dynamic> loc) {
    final speedMs = (loc['speed'] as num?)?.toDouble() ?? 0.0;
    final speedKmh = (speedMs * 3.6).clamp(0.0, 300.0);

    // If OBD telemetry is not streaming speed, use GPS speed
    if (_telemetry.speed == 0.0 || !_isBtConnected) {
      _telemetry = _telemetry.copyWith(speed: speedKmh);
      _updateTripTracking(speedKmh);
      notifyListeners();
    }
  }

  void _handleWifiState(Map<String, dynamic> event) {
    _isWifiConnected = event['connected'] as bool? ?? false;
    notifyListeners();
  }

  void _handleWsMessage(Map<String, dynamic> msg) {
    final type = msg['type'] as String?;
    if (type == 'bt_pin_code') {
      _pendingPinCode = msg['pin'] as String? ?? '';
      _pendingPinDeviceMac = msg['mac'] as String? ?? '';
      _pendingPinDeviceName = msg['name'] as String? ?? '';
      _addLog('[WS-PAIR] Yêu cầu mã xác nhận ghép nối PIN: $_pendingPinCode');
      notifyListeners();
    } else if (type == 'bt_status' || msg.containsKey('paired_devices')) {
      _handleBtStatusMessage(msg);
    } else if (type == 'bt_device_found') {
      final mac = (msg['mac'] as String? ?? '').toUpperCase();
      final name = (msg['name'] as String? ?? '').trim();
      final rssi = (msg['rssi'] as num?)?.toInt() ?? 0;
      if (mac.isNotEmpty) {
        final existing = _discoveredDevices[mac];
        String cleanName;
        if (!_isMacLikeName(name, mac)) {
          // If we receive a genuine friendly name, always use it!
          cleanName = name;
        } else if (existing != null && !_isMacLikeName(existing.name, mac)) {
          // Retain already resolved friendly name
          cleanName = existing.name;
        } else if (name.isNotEmpty && !_isMacLikeName(name, mac)) {
          cleanName = name;
        } else {
          final shortSuffix = mac.length >= 5 ? mac.substring(mac.length - 5) : mac;
          cleanName = 'Thiết bị ($shortSuffix)';
        }

        _discoveredDevices[mac] = BtDeviceInfo(
          mac: mac,
          name: cleanName,
          rssi: rssi,
          isBonded: _pairedDevices.any((d) => d.mac.toUpperCase() == mac),
        );
        notifyListeners();
      }
    } else if (type == 'bt_discovery_started') {
      _isDiscovering = true;
      notifyListeners();
    } else if (type == 'bt_discovery_stopped') {
      _isDiscovering = false;
      notifyListeners();
    } else if (type == 'bt_pair_result') {
      final mac = msg['mac'] as String? ?? '';
      final success = msg['success'] as bool? ?? false;
      final message = msg['message'] as String? ?? '';
      _addLog('[BT-PAIR] Ghép nối với $mac: ${success ? "Thành công" : "Thất bại ($message)"}');
      loadPiBluetoothStatus();
      notifyListeners();
    } else if (type == 'bt_logs' && msg['logs'] is List) {
      final logs = List<String>.from(msg['logs'] as List);
      for (final l in logs) {
        _addLog('[HUB-LOG] $l');
      }
      notifyListeners();
    } else if (type == 'bt_log_entry' && msg['line'] != null) {
      _addLog('[HUB-LOG] ${msg['line']}');
      notifyListeners();
    }
  }

  bool _isMacLikeName(String? name, String mac) {
    if (name == null || name.trim().isEmpty) return true;
    final cleanN = name.replaceAll(':', '').replaceAll('-', '').trim().toUpperCase();
    final cleanM = mac.replaceAll(':', '').replaceAll('-', '').trim().toUpperCase();
    if (cleanN == cleanM) return true;
    if (RegExp(r'^[0-9A-F]{12}$').hasMatch(cleanN)) return true;
    return false;
  }

  void _handleBtStatusMessage(Map<String, dynamic> msg) {
    final data = (msg['data'] as Map<String, dynamic>?) ?? msg;
    final paired = (data['paired_devices'] as List<dynamic>?) ?? [];
    _pairedDevices = paired.map((item) {
      final m = Map<String, dynamic>.from(item as Map);
      final mac = (m['mac'] as String? ?? '').toUpperCase();
      final rawName = (m['name'] as String? ?? '').trim();
      final shortSuffix = mac.length >= 5 ? mac.substring(mac.length - 5) : mac;
      final name = !_isMacLikeName(rawName, mac)
          ? rawName
          : 'Thiết bị Bluetooth ($shortSuffix)';
      return BtDeviceInfo(mac: mac, name: name, isBonded: true);
    }).toList();
    _addLog('[BT] Đã nhận danh sách ${_pairedDevices.length} thiết bị ghép nối');
    notifyListeners();
  }

  void _processIncomingMessage(String rawJson, String source) {
    try {
      final map = jsonDecode(rawJson) as Map<String, dynamic>;
      final type = map['type'] as String?;

      if (type == 'telemetry' || map.containsKey('speed') || map.containsKey('rpm')) {
        _telemetry = CarTelemetry.fromJson(map);
        _updateTripTracking(_telemetry.speed);
        notifyListeners();
      } else if (source == 'WS' && (type == 'bt_status' || map.containsKey('paired_devices'))) {
        _handleBtStatusMessage(map);
      } else if (source == 'WS' && type == 'bt_device_found') {
        _handleWsMessage(map);
      } else if (type == 'bt_pair_result') {
        _handleWsMessage(map);
      } else if (type == 'config' || map.containsKey('fl')) {
        _config = PiConfig.fromJson(map);
        storage.setFlId(_config.flSensorId);
        storage.setFrId(_config.frSensorId);
        storage.setRlId(_config.rlSensorId);
        storage.setRrId(_config.rrSensorId);
        storage.setAlertPressure(_config.pressureAlert);
        storage.setAlertTemp(_config.highTempAlert);
        notifyListeners();
      } else if (type == 'bt_logs' && map['logs'] is List) {
        final logs = List<String>.from(map['logs'] as List);
        for (final l in logs) {
          _addLog('[HUB-LOG] $l');
        }
        notifyListeners();
      } else if (type == 'bt_log_entry' && map['line'] != null) {
        _addLog('[HUB-LOG] ${map['line']}');
        notifyListeners();
      } else {
        _addLog('[$source] $rawJson');
      }
    } catch (_) {
      _addLog('[$source] $rawJson');
    }
  }

  void _addLog(String line) {
    final time = DateTime.now().toIso8601String().substring(11, 19);
    _btLogs.add('[$time] $line');
    if (_btLogs.length > 500) {
      _btLogs.removeAt(0);
    }
  }

  void clearLogs() {
    _btLogs.clear();
    notifyListeners();
  }

  void dismissPinDialog() {
    _pendingPinCode = null;
    _pendingPinDeviceMac = null;
    _pendingPinDeviceName = null;
    notifyListeners();
  }

  // --- Bluetooth Actions (Pi Hub Orchestrated) ---

  Future<void> loadPairedDevices() async {
    await loadPiBluetoothStatus();
  }

  Future<void> loadPiBluetoothStatus() async {
    final cmd = {'type': 'get_bt_status'};
    if (_isBtConnected) {
      sendBtCommand(cmd);
    }
    wifiWs.send(cmd);
  }

  Future<void> startDiscovery() async {
    _discoveredDevices.clear();
    _isDiscovering = true;
    notifyListeners();
    _addLog('[BT] Bắt đầu quét thiết bị Bluetooth...');
    final cmd = {'type': 'bt_start_discovery'};
    if (_isBtConnected) sendBtCommand(cmd);
    wifiWs.send(cmd);
  }

  Future<void> stopDiscovery() async {
    final cmd = {'type': 'bt_stop_discovery'};
    if (_isBtConnected) sendBtCommand(cmd);
    wifiWs.send(cmd);
    _isDiscovering = false;
    notifyListeners();
    _addLog('[BT] Dừng quét thiết bị Bluetooth');
  }

  Future<void> pairDevice(String mac) async {
    _addLog('[BT] Yêu cầu ghép nối thiết bị: $mac');
    final cmd = {'type': 'bt_pair', 'mac': mac};
    if (_isBtConnected) sendBtCommand(cmd);
    wifiWs.send(cmd);
  }

  Future<void> unpairDevice(String mac) async {
    _addLog('[BT] Yêu cầu hủy ghép nối thiết bị: $mac');
    final cmd = {'type': 'bt_unpair', 'mac': mac};
    if (_isBtConnected) sendBtCommand(cmd);
    wifiWs.send(cmd);
    await loadPiBluetoothStatus();
  }

  Future<void> connectToPi(String mac) async {
    if (mac.isEmpty) return;
    _reconnectTimer?.cancel();
    _isBtConnecting = true;
    notifyListeners();

    _addLog('[BT-SPP] Đang kết nối tới $mac...');
    storage.setPiMac(mac);

    final ok = await NativeBridge.connectBtSpp(mac);
    if (!ok) {
      _isBtConnecting = false;
      _scheduleAutoReconnect();
      notifyListeners();
    }
  }

  void disconnectPi() {
    _reconnectTimer?.cancel();
    _reconnectTimer = null;
    NativeBridge.disconnectBtSpp();
    _isBtConnected = false;
    _isBtConnecting = false;
    _connectedMac = '';
    _connectedName = '';
    notifyListeners();
  }

  bool sendBtCommand(Map<String, dynamic> cmd) {
    if (!_isBtConnected) return false;
    final jsonStr = jsonEncode(cmd);
    return NativeBridge.btSend(jsonStr) as dynamic ?? false;
  }

  void _scheduleAutoReconnect() {
    if (!storage.autoReconnect || storage.piMac.isEmpty) return;
    _reconnectTimer?.cancel();
    _reconnectTimer = Timer(const Duration(seconds: 5), () {
      if (!_isBtConnected && !_isBtConnecting && storage.piMac.isNotEmpty) {
        connectToPi(storage.piMac);
      }
    });
  }

  // --- Configuration ---

  Future<void> saveConfig(PiConfig newCfg) async {
    _config = newCfg;
    await storage.setFlId(newCfg.flSensorId);
    await storage.setFrId(newCfg.frSensorId);
    await storage.setRlId(newCfg.rlSensorId);
    await storage.setRrId(newCfg.rrSensorId);
    await storage.setAlertPressure(newCfg.pressureAlert);
    await storage.setAlertTemp(newCfg.highTempAlert);

    // ONLY send configuration over Bluetooth SPP as per user requirements!
    if (_isBtConnected) {
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
        }
      });
    }
    notifyListeners();
  }

  // --- Trip Tracking ---

  void _updateTripTracking(double speedKmh) {
    if (speedKmh > 5.0) {
      _tripIdleTimer?.cancel();
      _tripIdleTimer = null;

      if (_activeTrip == null) {
        _activeTrip = TripRecord(
          id: DateTime.now().millisecondsSinceEpoch.toString(),
          startTime: DateTime.now(),
        );
      } else {
        final now = DateTime.now();
        final durationSec = now.difference(_activeTrip!.startTime).inSeconds;
        final maxSpeed = speedKmh > _activeTrip!.maxSpeedKmh ? speedKmh : _activeTrip!.maxSpeedKmh;
        // Approximate distance: speed * time (1 sec)
        final distanceDelta = (speedKmh / 3600.0);
        final newDistance = _activeTrip!.distanceKm + distanceDelta;
        final avgSpeed = durationSec > 0 ? (newDistance / (durationSec / 3600.0)) : speedKmh;

        _activeTrip = _activeTrip!.copyWith(
          durationSeconds: durationSec,
          distanceKm: newDistance,
          maxSpeedKmh: maxSpeed,
          avgSpeedKmh: avgSpeed,
        );
      }
    } else if (_activeTrip != null) {
      // Stopped
      _tripIdleTimer ??= Timer(const Duration(minutes: 3), () {
        finishActiveTrip();
      });
    }
  }

  void finishActiveTrip() {
    if (_activeTrip == null) return;
    final finished = _activeTrip!.copyWith(endTime: DateTime.now());
    if (finished.distanceKm > 0.1) {
      _trips.insert(0, finished);
      storage.saveTrips(_trips);
    }
    _activeTrip = null;
    _tripIdleTimer?.cancel();
    _tripIdleTimer = null;
    notifyListeners();
  }

  void clearTrips() {
    _trips.clear();
    storage.saveTrips([]);
    notifyListeners();
  }

  @override
  void dispose() {
    _reconnectTimer?.cancel();
    _tripIdleTimer?.cancel();
    _btEventSub?.cancel();
    _locEventSub?.cancel();
    _wifiEventSub?.cancel();
    _wsMsgSub?.cancel();
    _wsConnSub?.cancel();
    wifiWs.dispose();
    super.dispose();
  }
}
