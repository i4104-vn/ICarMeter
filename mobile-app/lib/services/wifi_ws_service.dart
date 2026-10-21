import 'dart:async';
import 'dart:convert';
import 'dart:io';

class WifiWsService {
  WebSocket? _socket;
  String host;
  bool _isConnecting = false;
  Timer? _reconnectTimer;
  bool _manualDisconnect = false;

  final StreamController<Map<String, dynamic>> _messageController =
      StreamController<Map<String, dynamic>>.broadcast();
  final StreamController<bool> _connectionController =
      StreamController<bool>.broadcast();

  WifiWsService({this.host = '192.169.204.145:8080'});

  Stream<Map<String, dynamic>> get messages => _messageController.stream;
  Stream<bool> get connectionState => _connectionController.stream;

  bool get isConnected => _socket != null && _socket!.readyState == WebSocket.open;

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

      // Request BT status after connecting over Wi-Fi
      send({'type': 'get_bt_status'});
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

  void disconnect() {
    _manualDisconnect = true;
    _reconnectTimer?.cancel();
    _reconnectTimer = null;
    _socket?.close();
    _socket = null;
    _connectionController.add(false);
  }

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

  void dispose() {
    disconnect();
    _messageController.close();
    _connectionController.close();
  }
}
