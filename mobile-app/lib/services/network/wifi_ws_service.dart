import 'dart:async';
import 'dart:convert';
import 'dart:io';

/// Fallback WebSocket service for communicating with the Raspberry Pi over local Wi-Fi hotspot.
class WifiWsService {
  WebSocket? _socket;

  /// The target host and port (e.g. `192.168.4.1:8080`).
  String host;
  bool _isConnecting = false;
  Timer? _reconnectTimer;
  bool _manualDisconnect = false;

  final StreamController<Map<String, dynamic>> _messageController =
      StreamController<Map<String, dynamic>>.broadcast();
  final StreamController<bool> _connectionController =
      StreamController<bool>.broadcast();

  /// Creates a [WifiWsService] targeting the specified [host].
  WifiWsService({this.host = '192.168.4.1:8080'});

  /// Stream of decoded JSON message payloads received from the WebSocket server.
  Stream<Map<String, dynamic>> get messages => _messageController.stream;

  /// Stream of connection state updates (`true` when connected, `false` otherwise).
  Stream<bool> get connectionState => _connectionController.stream;

  /// Indicates whether the WebSocket connection is actively open.
  bool get isConnected => _socket != null && _socket!.readyState == WebSocket.open;

  /// Initiates an asynchronous connection to the WebSocket server.
  ///
  /// Automatically requests fresh Bluetooth status upon successful connection.
  void connect() async {
    if (isConnected || _isConnecting) return;
    _manualDisconnect = false;
    _isConnecting = true;

    try {
      final wsUrl = host.startsWith('ws://') || host.startsWith('wss://')
          ? host
          : 'ws://$host';
      _socket = await WebSocket.connect(wsUrl).timeout(const Duration(seconds: 4));
      _isConnecting = false;
      _connectionController.add(true);

      _socket!.listen(
        (data) {
          try {
            final parsed = jsonDecode(data as String) as Map<String, dynamic>;
            _messageController.add(parsed);
          } catch (_) {}
        },
        onDone: () {
          _socket = null;
          _connectionController.add(false);
          _scheduleReconnect();
        },
        onError: (err) {
          _socket = null;
          _connectionController.add(false);
          _scheduleReconnect();
        },
        cancelOnError: true,
      );

      // Request BT status and match/identification info after connecting over Wi-Fi
      send({'type': 'get_bt_status'});
      send({'type': 'match_pi'});
    } catch (_) {
      _socket = null;
      _isConnecting = false;
      _connectionController.add(false);
      _scheduleReconnect();
    }
  }

  void _scheduleReconnect() {
    if (_manualDisconnect) return;
    _reconnectTimer?.cancel();
    _reconnectTimer = Timer(const Duration(seconds: 4), () {
      if (!_manualDisconnect && !isConnected) {
        connect();
      }
    });
  }

  /// Manually closes the socket connection and suspends auto-reconnection.
  void disconnect() {
    _manualDisconnect = true;
    _reconnectTimer?.cancel();
    _reconnectTimer = null;
    _socket?.close();
    _socket = null;
    _connectionController.add(false);
  }

  /// Sends a JSON [command] map across the WebSocket.
  ///
  /// Telemetry and configuration commands are blocked from Wi-Fi transmission
  /// per the project requirements (must be transported via Bluetooth SPP).
  /// Returns `true` if transmission succeeded, or `false` otherwise.
  bool send(Map<String, dynamic> command) {
    final type = command['type'] as String?;
    // Strict requirement: No telemetry or configuration over Wi-Fi!
    if (type == 'get_config' || type == 'set_config' || type == 'telemetry') {
      return false;
    }

    if (!isConnected) return false;
    try {
      _socket?.add(jsonEncode(command));
      return true;
    } catch (_) {
      return false;
    }
  }

  /// Closes the connection and releases all active streams and timers.
  void dispose() {
    disconnect();
    _messageController.close();
    _connectionController.close();
  }
}
