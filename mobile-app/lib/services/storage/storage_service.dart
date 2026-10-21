import 'dart:convert';
import 'package:shared_preferences/shared_preferences.dart';
import '../../models/trip.dart';

/// Persistence service utilizing SharedPreferences for local settings and trips.
class StorageService {
  /// Preference key for the saved Raspberry Pi Bluetooth MAC address.
  static const String keyPiMac = 'pi_mac';

  /// Preference key for speedometer display units (`kmh` or `mph`).
  static const String keySpeedUnit = 'speed_unit';

  /// Preference key for tire pressure display units (`bar`, `psi`, or `kpa`).
  static const String keyPressureUnit = 'pressure_unit';

  /// Preference key for temperature display units (`C` or `F`).
  static const String keyTempUnit = 'temp_unit';

  /// Preference key for automatic Bluetooth SPP reconnection toggle.
  static const String keyAutoReconnect = 'auto_reconnect';

  /// Preference key for keeping the GPS service active in the background.
  static const String keyGpsForeground = 'gps_foreground';

  /// Preference key for the Front-Left TPMS sensor MAC address.
  static const String keyFlId = 'tpms_fl_id';

  /// Preference key for the Front-Right TPMS sensor MAC address.
  static const String keyFrId = 'tpms_fr_id';

  /// Preference key for the Rear-Left TPMS sensor MAC address.
  static const String keyRlId = 'tpms_rl_id';

  /// Preference key for the Rear-Right TPMS sensor MAC address.
  static const String keyRrId = 'tpms_rr_id';

  /// Preference key for the minimum tire pressure alert threshold in bar.
  static const String keyAlertPressure = 'tpms_alert_pressure';

  /// Preference key for the maximum tire temperature alert threshold in Celsius.
  static const String keyAlertTemp = 'tpms_alert_temp';

  /// Preference key for enabling or disabling the TPMS subsystem.
  static const String keyTpmsEnabled = 'tpms_enabled';

  /// Preference key for the serialized list of historical trips.
  static const String keyTrips = 'saved_trips';

  /// Underlying [SharedPreferences] instance.
  final SharedPreferences prefs;

  /// Creates a [StorageService] backed by the given [prefs].
  StorageService(this.prefs);

  /// Initializes and returns a ready-to-use [StorageService] asynchronously.
  static Future<StorageService> init() async {
    final sp = await SharedPreferences.getInstance();
    return StorageService(sp);
  }

  /// The persisted default Raspberry Pi Bluetooth MAC address.
  String get piMac => prefs.getString(keyPiMac) ?? '';

  /// Saves the Raspberry Pi Bluetooth [mac] address to persistent storage.
  Future<void> setPiMac(String mac) => prefs.setString(keyPiMac, mac);

  /// The active speed unit (`kmh` or `mph`).
  String get speedUnit => prefs.getString(keySpeedUnit) ?? 'kmh';

  /// Persists the active speed [unit] preference.
  Future<void> setSpeedUnit(String unit) => prefs.setString(keySpeedUnit, unit);

  /// The active tire pressure unit (`bar`, `psi`, or `kpa`).
  String get pressureUnit => prefs.getString(keyPressureUnit) ?? 'bar';

  /// Persists the tire pressure [unit] preference.
  Future<void> setPressureUnit(String unit) => prefs.setString(keyPressureUnit, unit);

  /// The active temperature unit (`C` or `F`).
  String get tempUnit => prefs.getString(keyTempUnit) ?? 'C';

  /// Persists the temperature [unit] preference.
  Future<void> setTempUnit(String unit) => prefs.setString(keyTempUnit, unit);

  /// Indicates whether the app should automatically reconnect to the vehicle over Bluetooth.
  bool get autoReconnect => prefs.getBool(keyAutoReconnect) ?? true;

  /// Sets whether Bluetooth auto-reconnect should be enabled.
  Future<void> setAutoReconnect(bool val) => prefs.setBool(keyAutoReconnect, val);

  /// Indicates whether the background GPS location service should run persistently.
  bool get gpsForeground => prefs.getBool(keyGpsForeground) ?? true;

  /// Sets whether background GPS positioning should remain active.
  Future<void> setGpsForeground(bool val) => prefs.setBool(keyGpsForeground, val);

  /// Sensor ID or MAC for the Front-Left tire.
  String get flId => prefs.getString(keyFlId) ?? '01:02:03:04:05:06';

  /// Saves the Front-Left TPMS sensor [id].
  Future<void> setFlId(String id) => prefs.setString(keyFlId, id);

  /// Sensor ID or MAC for the Front-Right tire.
  String get frId => prefs.getString(keyFrId) ?? '02:03:04:05:06:07';

  /// Saves the Front-Right TPMS sensor [id].
  Future<void> setFrId(String id) => prefs.setString(keyFrId, id);

  /// Sensor ID or MAC for the Rear-Left tire.
  String get rlId => prefs.getString(keyRlId) ?? '03:04:05:06:07:08';

  /// Saves the Rear-Left TPMS sensor [id].
  Future<void> setRlId(String id) => prefs.setString(keyRlId, id);

  /// Sensor ID or MAC for the Rear-Right tire.
  String get rrId => prefs.getString(keyRrId) ?? '04:05:06:07:08:09';

  /// Saves the Rear-Right TPMS sensor [id].
  Future<void> setRrId(String id) => prefs.setString(keyRrId, id);

  /// Low pressure threshold in bar below which warning indicators are triggered.
  double get alertPressure => prefs.getDouble(keyAlertPressure) ?? 1.8;

  /// Persists the low pressure alert threshold [val] in bar.
  Future<void> setAlertPressure(double val) => prefs.setDouble(keyAlertPressure, val);

  /// High temperature threshold in Celsius above which warning indicators are triggered.
  double get alertTemp => prefs.getDouble(keyAlertTemp) ?? 70.0;

  /// Persists the high temperature alert threshold [val] in Celsius.
  Future<void> setAlertTemp(double val) => prefs.setDouble(keyAlertTemp, val);

  /// Whether the TPMS subsystem is enabled locally.
  bool get tpmsEnabled => prefs.getBool(keyTpmsEnabled) ?? true;

  /// Persists the TPMS subsystem enabled state [val].
  Future<void> setTpmsEnabled(bool val) => prefs.setBool(keyTpmsEnabled, val);

  /// Loads and deserializes the list of saved trips from local storage.
  List<TripRecord> getTrips() {
    final raw = prefs.getString(keyTrips);
    if (raw == null || raw.isEmpty) return [];
    try {
      final list = jsonDecode(raw) as List;
      return list.map((e) => TripRecord.fromJson(e as Map<String, dynamic>)).toList();
    } catch (_) {
      return [];
    }
  }

  /// Serializes and saves the list of [trips] to persistent storage.
  Future<void> saveTrips(List<TripRecord> trips) async {
    final jsonList = trips.map((t) => t.toJson()).toList();
    await prefs.setString(keyTrips, jsonEncode(jsonList));
  }
}
