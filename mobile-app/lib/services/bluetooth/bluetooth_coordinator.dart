import 'dart:async';
import 'dart:convert';
import 'package:flutter/foundation.dart';
import '../../models/bluetooth_device.dart';
import '../storage/storage_service.dart';
import '../platform/native_bridge.dart';
import 'bt_log_service.dart';

/// Coordinator for Bluetooth state, discovery, pairing, and SPP socket communications.
class BluetoothCoordinator extends ChangeNotifier {
  /// Local persistence service for caching device MAC and reconnect settings.
  final StorageService storage;

  /// Logging service for capturing Bluetooth daemon events.
  final BtLogService logService;

  bool _isConnected = false;
  bool _isConnecting = false;
  String _connectedMac = '';
  String _connectedName = '';

  String _piMac = '';
  String _piName = 'ICarMeter';

  bool _isDiscovering = false;
  final Map<String, BtDeviceInfo> _discoveredDevices = {};
  List<BtDeviceInfo> _pairedDevices = [];

  String? _pendingPinCode;
  String? _pendingPinDeviceMac;
  String? _pendingPinDeviceName;

  Timer? _reconnectTimer;
  Timer? _tpmsPollTimer;

  /// Creates a [BluetoothCoordinator] with required dependencies.
  BluetoothCoordinator({
    required this.storage,
    required this.logService,
  });

  /// Indicates whether the RFCOMM Bluetooth SPP connection is actively open.
  bool get isConnected => _isConnected;

  /// Indicates whether a connection attempt is currently in progress.
  bool get isConnecting => _isConnecting;

  /// The MAC address of the currently connected Bluetooth peer.
  String get connectedMac => _connectedMac;

  /// The friendly name of the currently connected Bluetooth peer.
  String get connectedName => _connectedName;

  /// The MAC address of the Raspberry Pi vehicle hub.
  String get piMac => _piMac.isNotEmpty ? _piMac : storage.piMac;

  /// The friendly name of the Raspberry Pi vehicle hub.
  String get piName => _piName;

  /// Returns the Pi vehicle device info if known.
  BtDeviceInfo? get piDevice {
    final mac = piMac;
    if (mac.isEmpty) return null;
    return _discoveredDevices[mac] ??
        _pairedDevices.cast<BtDeviceInfo?>().firstWhere(
              (d) => d?.mac.toUpperCase() == mac.toUpperCase(),
              orElse: () => BtDeviceInfo(
                mac: mac,
                name: piName,
                isBonded: _pairedDevices.any((d) => d.mac.toUpperCase() == mac.toUpperCase()),
              ),
            );
  }

  /// Indicates whether device discovery scanning is currently active.
  bool get isDiscovering => _isDiscovering;

  /// List of newly discovered nearby Bluetooth devices.
  List<BtDeviceInfo> get discoveredDevices => _discoveredDevices.values.toList();

  /// List of devices currently paired and bonded with the vehicle hub.
  List<BtDeviceInfo> get pairedDevices => _pairedDevices;

  /// The generated PIN code awaiting user confirmation, or `null`.
  String? get pendingPinCode => _pendingPinCode;

  /// The MAC address of the device requesting PIN confirmation, or `null`.
  String? get pendingPinDeviceMac => _pendingPinDeviceMac;

  /// The name of the device requesting PIN confirmation, or `null`.
  String? get pendingPinDeviceName => _pendingPinDeviceName;

  /// Dismisses the pairing PIN confirmation dialog overlay.
  void dismissPinDialog() {
    _pendingPinCode = null;
    _pendingPinDeviceMac = null;
    _pendingPinDeviceName = null;
    notifyListeners();
  }

  /// Processes connection state changes emitted by the native Bluetooth SPP channel.
  void handleBtConnectionEvent(bool connected, String mac, String name) {
    _isConnected = connected;
    _isConnecting = false;
    if (connected) {
      _connectedMac = mac;
      _connectedName = name.isNotEmpty ? name : 'Thiết bị trên xe';
      logService.addLog('[BT-SPP] Đã kết nối với $_connectedName ($_connectedMac)');

      sendBtCommand({'type': 'get_config'});
      sendBtCommand({'type': 'get_tpms'});
      sendBtCommand({'type': 'get_bt_logs'});
      _startTpmsPollTimer();
    } else {
      logService.addLog('[BT-SPP] Đã ngắt kết nối Bluetooth SPP');
      _connectedMac = '';
      _connectedName = '';
      _stopTpmsPollTimer();
      _scheduleAutoReconnect();
    }
    notifyListeners();
  }

  bool _isMacLikeName(String? name, String mac) {
    if (name == null || name.trim().isEmpty) return true;
    final cleanN = name.replaceAll(':', '').replaceAll('-', '').trim().toUpperCase();
    final cleanM = mac.replaceAll(':', '').replaceAll('-', '').trim().toUpperCase();
    if (cleanN == cleanM) return true;
    if (RegExp(r'^[0-9A-F]{12}$').hasMatch(cleanN)) return true;
    return false;
  }

  /// Handles discovery of the Raspberry Pi vehicle hub over Bluetooth.
  void handlePiDiscovered(String mac, String name) {
    if (mac.isEmpty) return;
    final cleanMac = mac.trim().toUpperCase();
    final cleanName = name.trim().isNotEmpty ? name.trim() : 'ICarMeter';
    _piMac = cleanMac;
    _piName = cleanName;
    storage.setPiMac(cleanMac);

    final isBonded = _pairedDevices.any((d) => d.mac.toUpperCase() == cleanMac);
    _discoveredDevices[cleanMac] = BtDeviceInfo(
      mac: cleanMac,
      name: cleanName,
      rssi: 0,
      isBonded: isBonded,
    );

    logService.addLog('[MATCH] Đã nhận diện xe $cleanName ($cleanMac)');
    notifyListeners();

    // Auto-connect if auto-reconnect is enabled and not yet connected/connecting
    if (storage.autoReconnect && !_isConnected && !_isConnecting) {
      logService.addLog('[MATCH] Tự động kết nối Bluetooth với xe $cleanMac...');
      connectToPi(cleanMac);
    }
  }

  /// Parses and stores the updated list of paired devices received from the hub.
  void handleBtStatusMessage(Map<String, dynamic> msg) {
    final data = (msg['data'] as Map<String, dynamic>?) ?? msg;

    // Check if Pi's own Bluetooth MAC was included
    final hubMac = ((data['pi_mac'] ?? msg['pi_mac']) as String? ?? '').trim().toUpperCase();
    if (hubMac.isNotEmpty) {
      final hubName = ((data['pi_name'] ?? msg['pi_name']) as String? ?? 'ICarMeter').trim();
      handlePiDiscovered(hubMac, hubName);
    }

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
    logService.addLog('[BT] Đã nhận danh sách ${_pairedDevices.length} thiết bị ghép nối');
    notifyListeners();
  }

  /// Records or updates a newly discovered Bluetooth device in [_discoveredDevices].
  void handleDeviceFound(Map<String, dynamic> msg) {
    final mac = (msg['mac'] as String? ?? '').toUpperCase();
    final name = (msg['name'] as String? ?? '').trim();
    final rssi = (msg['rssi'] as num?)?.toInt() ?? 0;
    if (mac.isEmpty) return;

    final existing = _discoveredDevices[mac];
    final hasFriendlyName = !_isMacLikeName(name, mac);
    final hasExistingFriendly = existing != null && !_isMacLikeName(existing.name, mac);

    final cleanName = switch ((hasFriendlyName, hasExistingFriendly)) {
      (true, _) => name,
      (false, true) => existing!.name,
      _ => (name.isNotEmpty && !_isMacLikeName(name, mac))
          ? name
          : 'Thiết bị (${mac.length >= 5 ? mac.substring(mac.length - 5) : mac})',
    };

    _discoveredDevices[mac] = BtDeviceInfo(
      mac: mac,
      name: cleanName,
      rssi: rssi,
      isBonded: _pairedDevices.any((d) => d.mac.toUpperCase() == mac),
    );
    notifyListeners();
  }

  /// Activates the pairing PIN confirmation prompt with the given [pin], [mac], and [name].
  void handlePinRequest(String pin, String mac, String name) {
    _pendingPinCode = pin;
    _pendingPinDeviceMac = mac;
    _pendingPinDeviceName = name;
    logService.addLog('[BT-PAIR] PIN/passkey requested: $_pendingPinCode');
    notifyListeners();
  }

  /// Updates discovery scanning state and notifies listeners.
  void setDiscoveryState(bool active) {
    _isDiscovering = active;
    notifyListeners();
  }

  /// Sends a command to fetch the latest paired devices list from the hub daemon.
  Future<void> loadPairedDevices() async {
    final cmd = {'type': 'get_bt_status'};
    if (_isConnected) await sendBtCommand(cmd);
  }

  /// Sends a matching command to identify and pair with the Raspberry Pi.
  Future<void> matchWithPi() async {
    logService.addLog('[MATCH] Connecting to the vehicle over Bluetooth...');
    final mac = piMac;
    if (mac.isNotEmpty && !_isConnected && !_isConnecting) {
      await connectToPi(mac);
    } else if (_isConnected) {
      await sendBtCommand({'type': 'get_bt_status'});
      await sendBtCommand({'type': 'match_pi'});
    }
  }

  /// Starts the phone's native Bluetooth discovery scan and, when connected,
  /// also asks the vehicle hub to scan its local Bluetooth environment.
  Future<void> startDiscovery() async {
    _discoveredDevices.clear();
    _isDiscovering = true;
    notifyListeners();
    logService.addLog('[BT] Bắt đầu quét thiết bị Bluetooth...');
    await NativeBridge.startBtDiscovery();
    final cmd = {'type': 'bt_start_discovery'};
    if (_isConnected) await sendBtCommand(cmd);
  }

  /// Halts the active Bluetooth discovery scan.
  Future<void> stopDiscovery() async {
    await NativeBridge.stopBtDiscovery();
    final cmd = {'type': 'bt_stop_discovery'};
    if (_isConnected) await sendBtCommand(cmd);
    _isDiscovering = false;
    notifyListeners();
    logService.addLog('[BT] Dừng quét thiết bị Bluetooth');
  }

  /// Dispatches a pairing request for the specified [mac] address.
  Future<void> pairDevice(String mac) async {
    logService.addLog('[BT] Yêu cầu ghép nối thiết bị: $mac');
    final cmd = {'type': 'bt_pair', 'mac': mac};
    if (_isConnected) await sendBtCommand(cmd);
  }

  /// Dispatches an unpair request to remove the bond with [mac].
  Future<void> unpairDevice(String mac) async {
    logService.addLog('[BT] Yêu cầu hủy ghép nối thiết bị: $mac');
    final cmd = {'type': 'bt_unpair', 'mac': mac};
    if (_isConnected) await sendBtCommand(cmd);
    await loadPairedDevices();
  }

  /// Connects to the Raspberry Pi Bluetooth SPP service using [mac].
  ///
  /// Caches the MAC address for future auto-reconnections.
  Future<void> connectToPi(String mac) async {
    if (mac.isEmpty) return;
    _reconnectTimer?.cancel();
    _isConnecting = true;
    notifyListeners();

    logService.addLog('[BT-SPP] Đang kết nối tới $mac...');
    storage.setPiMac(mac);

    final ok = await NativeBridge.connectBtSpp(mac);
    if (!ok) {
      _isConnecting = false;
      _scheduleAutoReconnect();
      notifyListeners();
    }
  }

  /// Disconnects the active Bluetooth SPP session and cancels reconnection timers.
  void disconnectPi() {
    _reconnectTimer?.cancel();
    _reconnectTimer = null;
    NativeBridge.disconnectBtSpp();
    _isConnected = false;
    _isConnecting = false;
    _connectedMac = '';
    _connectedName = '';
    notifyListeners();
  }

  /// Sends a JSON [cmd] across the active Bluetooth SPP socket.
  ///
  /// Returns `true` if transmission succeeded, or `false` otherwise.
  Future<bool> sendBtCommand(Map<String, dynamic> cmd) async {
    if (!_isConnected) return false;
    final jsonStr = jsonEncode(cmd);
    return NativeBridge.btSend(jsonStr);
  }

  /// Requests the latest TPMS sensor configuration and readings over Bluetooth.
  void syncTpms() {
    if (_isConnected) {
      sendBtCommand({'type': 'get_config'});
      sendBtCommand({'type': 'get_tpms'});
    }
  }

  void _scheduleAutoReconnect() {
    if (!storage.autoReconnect || storage.piMac.isEmpty) return;
    _reconnectTimer?.cancel();
    _reconnectTimer = Timer(const Duration(seconds: 5), () {
      if (!_isConnected && !_isConnecting && storage.piMac.isNotEmpty) {
        connectToPi(storage.piMac);
      }
    });
  }

  void _startTpmsPollTimer() {
    _tpmsPollTimer?.cancel();
    _tpmsPollTimer = Timer.periodic(const Duration(seconds: 2), (_) {
      if (_isConnected) {
        sendBtCommand({'type': 'get_tpms'});
      }
    });
  }

  void _stopTpmsPollTimer() {
    _tpmsPollTimer?.cancel();
    _tpmsPollTimer = null;
  }

  /// Cancels polling and reconnection timers and cleans up resources.
  @override
  void dispose() {
    _reconnectTimer?.cancel();
    _tpmsPollTimer?.cancel();
    super.dispose();
  }
}
