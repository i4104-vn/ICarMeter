import 'dart:convert';
import 'package:shared_preferences/shared_preferences.dart';
import '../models/trip.dart';

class StorageService {
  static const String keyPiMac = 'pi_mac';
  static const String keySpeedUnit = 'speed_unit';
  static const String keyPressureUnit = 'pressure_unit';
  static const String keyTempUnit = 'temp_unit';
  static const String keyAutoReconnect = 'auto_reconnect';
  static const String keyGpsForeground = 'gps_foreground';
  static const String keyFlId = 'tpms_fl_id';
  static const String keyFrId = 'tpms_fr_id';
  static const String keyRlId = 'tpms_rl_id';
  static const String keyRrId = 'tpms_rr_id';
  static const String keyAlertPressure = 'tpms_alert_pressure';
  static const String keyAlertTemp = 'tpms_alert_temp';
  static const String keyTrips = 'saved_trips';

  final SharedPreferences prefs;

  StorageService(this.prefs);

  static Future<StorageService> init() async {
    final sp = await SharedPreferences.getInstance();
    return StorageService(sp);
  }

  String get piMac => prefs.getString(keyPiMac) ?? '';
  Future<void> setPiMac(String mac) => prefs.setString(keyPiMac, mac);

  String get speedUnit => prefs.getString(keySpeedUnit) ?? 'kmh';
  Future<void> setSpeedUnit(String unit) => prefs.setString(keySpeedUnit, unit);

  String get pressureUnit => prefs.getString(keyPressureUnit) ?? 'bar';
  Future<void> setPressureUnit(String unit) => prefs.setString(keyPressureUnit, unit);

  String get tempUnit => prefs.getString(keyTempUnit) ?? 'C';
  Future<void> setTempUnit(String unit) => prefs.setString(keyTempUnit, unit);

  bool get autoReconnect => prefs.getBool(keyAutoReconnect) ?? true;
  Future<void> setAutoReconnect(bool val) => prefs.setBool(keyAutoReconnect, val);

  bool get gpsForeground => prefs.getBool(keyGpsForeground) ?? true;
  Future<void> setGpsForeground(bool val) => prefs.setBool(keyGpsForeground, val);

  String get flId => prefs.getString(keyFlId) ?? '01:02:03:04:05:06';
  Future<void> setFlId(String id) => prefs.setString(keyFlId, id);

  String get frId => prefs.getString(keyFrId) ?? '02:03:04:05:06:07';
  Future<void> setFrId(String id) => prefs.setString(keyFrId, id);

  String get rlId => prefs.getString(keyRlId) ?? '03:04:05:06:07:08';
  Future<void> setRlId(String id) => prefs.setString(keyRlId, id);

  String get rrId => prefs.getString(keyRrId) ?? '04:05:06:07:08:09';
  Future<void> setRrId(String id) => prefs.setString(keyRrId, id);

  double get alertPressure => prefs.getDouble(keyAlertPressure) ?? 1.8;
  Future<void> setAlertPressure(double val) => prefs.setDouble(keyAlertPressure, val);

  double get alertTemp => prefs.getDouble(keyAlertTemp) ?? 70.0;
  Future<void> setAlertTemp(double val) => prefs.setDouble(keyAlertTemp, val);

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

  Future<void> saveTrips(List<TripRecord> trips) async {
    final jsonList = trips.map((t) => t.toJson()).toList();
    await prefs.setString(keyTrips, jsonEncode(jsonList));
  }
}
