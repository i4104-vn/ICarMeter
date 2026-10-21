class PiConfig {
  final String flSensorId;
  final String frSensorId;
  final String rlSensorId;
  final String rrSensorId;
  final double pressureAlert; // bar (e.g. 1.8)
  final double highTempAlert; // Celsius (e.g. 70.0)
  final bool gpsEnabled;

  const PiConfig({
    this.flSensorId = 'A4:C1:38:00:00:01',
    this.frSensorId = 'A4:C1:38:00:00:02',
    this.rlSensorId = 'A4:C1:38:00:00:03',
    this.rrSensorId = 'A4:C1:38:00:00:04',
    this.pressureAlert = 2.2,
    this.highTempAlert = 70.0,
    this.gpsEnabled = true,
  });

  factory PiConfig.fromJson(Map<String, dynamic> json) {
    Map<String, dynamic> source = json;
    if (json['data'] is Map<String, dynamic>) {
      source = json['data'] as Map<String, dynamic>;
    }
    Map<String, dynamic> tpmsMap = source;
    if (source['tpms'] is Map<String, dynamic>) {
      tpmsMap = source['tpms'] as Map<String, dynamic>;
    }

    return PiConfig(
      flSensorId: tpmsMap['fl'] as String? ??
          tpmsMap['fl_sensor'] as String? ??
          tpmsMap['flSensorId'] as String? ??
          '',
      frSensorId: tpmsMap['fr'] as String? ??
          tpmsMap['fr_sensor'] as String? ??
          tpmsMap['frSensorId'] as String? ??
          '',
      rlSensorId: tpmsMap['rl'] as String? ??
          tpmsMap['rl_sensor'] as String? ??
          tpmsMap['rlSensorId'] as String? ??
          '',
      rrSensorId: tpmsMap['rr'] as String? ??
          tpmsMap['rr_sensor'] as String? ??
          tpmsMap['rrSensorId'] as String? ??
          '',
      pressureAlert: (tpmsMap['alert'] as num?)?.toDouble() ??
          (tpmsMap['low_pressure_alert'] as num?)?.toDouble() ??
          (tpmsMap['pressureAlert'] as num?)?.toDouble() ??
          2.2,
      highTempAlert: (tpmsMap['high_temp_alert'] as num?)?.toDouble() ??
          (tpmsMap['highTempAlert'] as num?)?.toDouble() ??
          70.0,
      gpsEnabled: (source['gps_enabled'] as bool?) ??
          ((source['gps'] is Map<String, dynamic>)
              ? (source['gps'] as Map<String, dynamic>)['enabled'] as bool? ?? true
              : true),
    );
  }

  Map<String, dynamic> toJson() => {
        'fl': flSensorId,
        'fr': frSensorId,
        'rl': rlSensorId,
        'rr': rrSensorId,
        'alert': pressureAlert,
        'high_temp_alert': highTempAlert,
        'gps_enabled': gpsEnabled,
      };

  PiConfig copyWith({
    String? flSensorId,
    String? frSensorId,
    String? rlSensorId,
    String? rrSensorId,
    double? pressureAlert,
    double? highTempAlert,
    bool? gpsEnabled,
  }) {
    return PiConfig(
      flSensorId: flSensorId ?? this.flSensorId,
      frSensorId: frSensorId ?? this.frSensorId,
      rlSensorId: rlSensorId ?? this.rlSensorId,
      rrSensorId: rrSensorId ?? this.rrSensorId,
      pressureAlert: pressureAlert ?? this.pressureAlert,
      highTempAlert: highTempAlert ?? this.highTempAlert,
      gpsEnabled: gpsEnabled ?? this.gpsEnabled,
    );
  }
}
