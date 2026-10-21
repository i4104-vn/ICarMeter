import 'dart:convert';
import '../../models/telemetry.dart';
import '../../models/config.dart';

/// Processor and dispatcher for incoming JSON messages from Bluetooth SPP and WebSocket.
class IncomingMessageDispatcher {
  /// Callback invoked when a TPMS packet is parsed.
  final void Function(TpmsData tpms) onTpms;

  /// Callback invoked when vehicle OBD/speed telemetry is parsed.
  final void Function(CarTelemetry telemetry) onTelemetry;

  /// Callback invoked when vehicle/sensor hardware configuration is parsed.
  final void Function(PiConfig config) onConfig;

  /// Callback invoked when a Bluetooth status/paired devices list is received.
  final void Function(Map<String, dynamic> status) onBtStatus;

  /// Callback invoked when a new nearby Bluetooth device is discovered.
  final void Function(Map<String, dynamic> device) onBtDeviceFound;

  /// Callback invoked when a pairing attempt completes with success/failure.
  final void Function(Map<String, dynamic> result) onBtPairResult;

  /// Callback invoked when a batch of hub daemon logs is received.
  final void Function(List<String> logs) onBtLogs;

  /// Callback invoked when a single log entry line arrives.
  final void Function(String line) onBtLogEntry;

  /// Callback invoked for unparsed raw log messages or parse exceptions.
  final void Function(String line) onLog;

  /// Creates an [IncomingMessageDispatcher] with required typed message handlers.
  IncomingMessageDispatcher({
    required this.onTpms,
    required this.onTelemetry,
    required this.onConfig,
    required this.onBtStatus,
    required this.onBtDeviceFound,
    required this.onBtPairResult,
    required this.onBtLogs,
    required this.onBtLogEntry,
    required this.onLog,
  });

  /// Decodes and routes a raw JSON message string from the specified [source] (`'BT'` or `'WS'`).
  ///
  /// Dispatches to the appropriate typed callback using pattern matching on the `type` field.
  /// If the `type` field is absent, fallback heuristic detection is applied for legacy payloads.
  void process(String rawJson, String source) {
    try {
      final map = jsonDecode(rawJson) as Map<String, dynamic>;
      final type = map['type'] as String?;

      switch (type) {
        case 'tpms':
          final tpmsJson = map['data'] is Map<String, dynamic>
              ? map['data'] as Map<String, dynamic>
              : (map['tpms'] is Map<String, dynamic> ? map['tpms'] as Map<String, dynamic> : map);
          onTpms(TpmsData.fromJson(tpmsJson));
          break;

        case 'telemetry':
          onTelemetry(CarTelemetry.fromJson(map));
          break;

        case 'bt_status':
          if (source == 'WS') {
            onBtStatus(map);
          }
          break;

        case 'bt_device_found':
          if (source == 'WS') {
            onBtDeviceFound(map);
          }
          break;

        case 'bt_pair_result':
          onBtPairResult(map);
          break;

        case 'config':
          onConfig(PiConfig.fromJson(map));
          break;

        case 'bt_logs':
          if (map['logs'] is List) {
            final logs = List<String>.from(map['logs'] as List);
            onBtLogs(logs);
          }
          break;

        case 'bt_log_entry':
          if (map['line'] != null) {
            onBtLogEntry(map['line'] as String);
          }
          break;

        default:
          // Fallback parsing for legacy payloads without explicit 'type'
          switch (true) {
            case true when map.containsKey('speed') || map.containsKey('rpm'):
              onTelemetry(CarTelemetry.fromJson(map));
              break;

            case true when source == 'WS' && map.containsKey('paired_devices'):
              onBtStatus(map);
              break;

            case true when map.containsKey('fl'):
              onConfig(PiConfig.fromJson(map));
              break;

            default:
              onLog('[$source] $rawJson');
              break;
          }
          break;
      }
    } catch (_) {
      onLog('[$source] $rawJson');
    }
  }
}
