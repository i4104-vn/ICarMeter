class TripRecord {
  final String id;
  final DateTime startTime;
  final DateTime? endTime;
  final double distanceKm;
  final double maxSpeedKmh;
  final double avgSpeedKmh;
  final int durationSeconds;
  final double fuelConsumedLiters;

  const TripRecord({
    required this.id,
    required this.startTime,
    this.endTime,
    this.distanceKm = 0.0,
    this.maxSpeedKmh = 0.0,
    this.avgSpeedKmh = 0.0,
    this.durationSeconds = 0,
    this.fuelConsumedLiters = 0.0,
  });

  factory TripRecord.fromJson(Map<String, dynamic> json) {
    return TripRecord(
      id: json['id'] as String? ?? DateTime.now().millisecondsSinceEpoch.toString(),
      startTime: DateTime.tryParse(json['startTime'] as String? ?? '') ?? DateTime.now(),
      endTime: json['endTime'] != null ? DateTime.tryParse(json['endTime'] as String) : null,
      distanceKm: (json['distanceKm'] as num?)?.toDouble() ?? 0.0,
      maxSpeedKmh: (json['maxSpeedKmh'] as num?)?.toDouble() ?? 0.0,
      avgSpeedKmh: (json['avgSpeedKmh'] as num?)?.toDouble() ?? 0.0,
      durationSeconds: (json['durationSeconds'] as num?)?.toInt() ?? 0,
      fuelConsumedLiters: (json['fuelConsumedLiters'] as num?)?.toDouble() ?? 0.0,
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'startTime': startTime.toIso8601String(),
        'endTime': endTime?.toIso8601String(),
        'distanceKm': distanceKm,
        'maxSpeedKmh': maxSpeedKmh,
        'avgSpeedKmh': avgSpeedKmh,
        'durationSeconds': durationSeconds,
        'fuelConsumedLiters': fuelConsumedLiters,
      };

  TripRecord copyWith({
    String? id,
    DateTime? startTime,
    DateTime? endTime,
    double? distanceKm,
    double? maxSpeedKmh,
    double? avgSpeedKmh,
    int? durationSeconds,
    double? fuelConsumedLiters,
  }) {
    return TripRecord(
      id: id ?? this.id,
      startTime: startTime ?? this.startTime,
      endTime: endTime ?? this.endTime,
      distanceKm: distanceKm ?? this.distanceKm,
      maxSpeedKmh: maxSpeedKmh ?? this.maxSpeedKmh,
      avgSpeedKmh: avgSpeedKmh ?? this.avgSpeedKmh,
      durationSeconds: durationSeconds ?? this.durationSeconds,
      fuelConsumedLiters: fuelConsumedLiters ?? this.fuelConsumedLiters,
    );
  }
}
