import 'dart:convert';

import '../../models/config.dart';
import '../../models/telemetry.dart';

/// Processor and dispatcher for incoming Bluetooth SPP JSON messages.
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

  /// Callback invoked when Bluetooth pairing requests PIN / passkey confirmation.
  final void Function(String pin, String mac, String name) onBtPinCode;

  /// Callback invoked when Bluetooth discovery starts or stops.
  final void Function(bool active) onBtDiscoveryState;

  /// Callback invoked when the vehicle hub is identified.
  final void Function(String mac, String name) onPiDiscovered;

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
    required this.onBtPinCode,
    required this.onBtDiscoveryState,
    required this.onPiDiscovered,
    required this.onLog,
  });

  /// Decodes and routes a raw JSON message string received over Bluetooth SPP.
  ///
  /// Dispatches to the appropriate typed callback using pattern matching on the `type` field.
  /// If the `type` field is absent, fallback heuristic detection is applied for legacy payloads.
  void process(String rawJson) {
    try {
      final map = jsonDecode(rawJson) as Map<String, dynamic>;
      final type = map['type'] as String?;

      switch (type) {
        case 'welcome':
          if (map['config'] is Map<String, dynamic>) {
            onConfig(PiConfig.fromJson(map['config'] as Map<String, dynamic>));
          }
          final piMac = (map['pi_mac'] as String? ?? '').trim().toUpperCase();
          if (piMac.isNotEmpty) {
            final piName = (map['pi_name'] as String? ?? 'ICarMeter').trim();
            onPiDiscovered(piMac, piName.isNotEmpty ? piName : 'ICarMeter');
          }
          break;

        case 'tpms':
          final tpmsJson = map['data'] is Map<String, dynamic>
              ? map['data'] as Map<String, dynamic>
              : (map['tpms'] is Map<String, dynamic>
                  ? map['tpms'] as Map<String, dynamic>
                  : map);
          onTpms(TpmsData.fromJson(tpmsJson));
          break;

        case 'telemetry':
          onTelemetry(CarTelemetry.fromJson(map));
          break;

        case 'bt_pin_code':
          onBtPinCode(
            map['pin'] as String? ?? '',
            map['mac'] as String? ?? '',
            map['name'] as String? ?? '',
          );
          break;

        case 'bt_status':
          final data = (map['data'] as Map<String, dynamic>?) ?? map;
          final piMac = ((data['pi_mac'] ?? map['pi_mac']) as String? ?? '')
              .trim()
              .toUpperCase();
          if (piMac.isNotEmpty) {
            final piName =
                ((data['pi_name'] ?? map['pi_name']) as String? ?? 'ICarMeter')
                    .trim();
            onPiDiscovered(piMac, piName.isNotEmpty ? piName : 'ICarMeter');
          }
          onBtStatus(map);
          break;

        case 'pi_matched':
        case 'pi_info':
          final piMac = (map['pi_mac'] as String? ?? '').trim().toUpperCase();
          if (piMac.isNotEmpty) {
            final piName = (map['pi_name'] as String? ?? 'ICarMeter').trim();
            onPiDiscovered(piMac, piName.isNotEmpty ? piName : 'ICarMeter');
          }
          break;

        case 'bt_discovery_started':
          onBtDiscoveryState(true);
          break;

        case 'bt_discovery_stopped':
          onBtDiscoveryState(false);
          break;

        case 'bt_device_found':
          onBtDeviceFound(map);
          break;

        case 'bt_pair_result':
          onBtPairResult(map);
          break;

        case 'config':
          final configJson = map['data'] is Map<String, dynamic>
              ? map['data'] as Map<String, dynamic>
              : map;
          onConfig(PiConfig.fromJson(configJson));
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

            case true when map.containsKey('paired_devices'):
              onBtStatus(map);
              break;

            case true when map.containsKey('fl'):
              onConfig(PiConfig.fromJson(map));
              break;

            default:
              onLog('[BT] $rawJson');
              break;
          }
          break;
      }
    } catch (_) {
      onLog('[BT] $rawJson');
    }
  }
}
