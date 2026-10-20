class PiConfig {
  final String flSensorId;
  final String frSensorId;
  final String rlSensorId;
  final String rrSensorId;
  final double pressureAlert; // bar (e.g. 1.8)
  final double highTempAlert; // Celsius (e.g. 70.0)
  final bool gpsEnabled;

  const PiConfig({
    this.flSensorId = '01:02:03:04:05:06',
    this.frSensorId = '02:03:04:05:06:07',
    this.rlSensorId = '03:04:05:06:07:08',
    this.rrSensorId = '04:05:06:07:08:09',
    this.pressureAlert = 1.8,
    this.highTempAlert = 70.0,
    this.gpsEnabled = true,
  });

  factory PiConfig.fromJson(Map<String, dynamic> json) {
    return PiConfig(
      flSensorId: json['fl'] as String? ?? json['flSensorId'] as String? ?? '',
      frSensorId: json['fr'] as String? ?? json['frSensorId'] as String? ?? '',
      rlSensorId: json['rl'] as String? ?? json['rlSensorId'] as String? ?? '',
      rrSensorId: json['rr'] as String? ?? json['rrSensorId'] as String? ?? '',
      pressureAlert: (json['alert'] as num?)?.toDouble() ??
          (json['pressureAlert'] as num?)?.toDouble() ??
          1.8,
      highTempAlert: (json['high_temp_alert'] as num?)?.toDouble() ??
          (json['highTempAlert'] as num?)?.toDouble() ??
          70.0,
      gpsEnabled: (json['gps_enabled'] as bool?) ?? true,
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
