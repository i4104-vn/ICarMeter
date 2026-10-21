import 'package:flutter/foundation.dart';

/// Buffer service managing timestamped Bluetooth daemon and connection logs.
class BtLogService extends ChangeNotifier {
  final List<String> _logs = [];

  /// An unmodifiable view of all recorded log entries.
  List<String> get logs => List.unmodifiable(_logs);

  /// Formats and appends a single [line] with current time `[HH:mm:ss]`.
  ///
  /// Maintains a maximum ring buffer capacity of 500 entries by removing the
  /// oldest log when exceeded.
  void addLog(String line) {
    final time = DateTime.now().toIso8601String().substring(11, 19);
    _logs.add('[$time] $line');
    if (_logs.length > 500) {
      _logs.removeAt(0);
    }
    notifyListeners();
  }

  /// Appends multiple hub daemon [lines] prefixed with `[HUB-LOG]`.
  ///
  /// Each entry is timestamped with the current time and constrained by the
  /// 500-entry capacity limit.
  void addLogs(List<String> lines) {
    final time = DateTime.now().toIso8601String().substring(11, 19);
    for (final l in lines) {
      _logs.add('[$time] [HUB-LOG] $l');
    }
    while (_logs.length > 500) {
      _logs.removeAt(0);
    }
    notifyListeners();
  }

  /// Clears all recorded log entries and notifies listeners.
  void clear() {
    _logs.clear();
    notifyListeners();
  }
}
