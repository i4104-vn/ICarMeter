class TireData {
  final double pressureBar;
  final double tempC;
  final int batteryPercent;
  final String status; // 'ok', 'low_pressure', 'high_pressure', 'disconnected'

  const TireData({
    this.pressureBar = 0.0,
    this.tempC = 0.0,
    this.batteryPercent = 100,
    this.status = 'disconnected',
  });

  bool get isDisconnected => pressureBar <= 0.0;
  bool isLow(double alertThreshold) => pressureBar > 0.0 && pressureBar < alertThreshold;
  bool isHighTemp(double alertTemp) => tempC >= alertTemp;

  factory TireData.fromJson(Map<String, dynamic>? json) {
    if (json == null) return const TireData();
    return TireData(
      pressureBar: (json['pressure'] as num?)?.toDouble() ??
          (json['pressureBar'] as num?)?.toDouble() ??
          (json['bar'] as num?)?.toDouble() ??
          0.0,
      tempC: (json['temperature'] as num?)?.toDouble() ??
          (json['tempC'] as num?)?.toDouble() ??
          (json['temp'] as num?)?.toDouble() ??
          0.0,
      batteryPercent: (json['battery'] as num?)?.toInt() ?? 100,
      status: json['status'] as String? ?? 'ok',
    );
  }

  Map<String, dynamic> toJson() => {
        'pressure': pressureBar,
        'temperature': tempC,
        'battery': batteryPercent,
        'status': status,
      };
}

class TpmsData {
  final TireData fl;
  final TireData fr;
  final TireData rl;
  final TireData rr;

  const TpmsData({
    this.fl = const TireData(),
    this.fr = const TireData(),
    this.rl = const TireData(),
    this.rr = const TireData(),
  });

  factory TpmsData.fromJson(Map<String, dynamic>? json) {
    if (json == null) return const TpmsData();

    TireData parseTire(String key, String tempKey) {
      final val = json[key];
      if (val is Map<String, dynamic>) {
        return TireData.fromJson(val);
      }
      final p = (val as num?)?.toDouble() ??
          (json['${key}_pressure'] as num?)?.toDouble() ??
          0.0;
      final t = (json[tempKey] as num?)?.toDouble() ?? 0.0;
      return TireData(pressureBar: p, tempC: t);
    }

    return TpmsData(
      fl: parseTire('fl', 'fl_temp'),
      fr: parseTire('fr', 'fr_temp'),
      rl: parseTire('rl', 'rl_temp'),
      rr: parseTire('rr', 'rr_temp'),
    );
  }

  Map<String, dynamic> toJson() => {
        'fl': fl.toJson(),
        'fr': fr.toJson(),
        'rl': rl.toJson(),
        'rr': rr.toJson(),
      };
}

class CarTelemetry {
  final double speed; // km/h
  final double rpm;
  final double coolantTemp; // Celsius
  final double fuelLevel; // 0..100%
  final double batteryVoltage; // Volts
  final double engineLoad; // 0..100%
  final double throttlePos; // 0..100%
  final double intakeTemp; // Celsius
  final double ambientTemp; // Celsius
  final double odometer; // km
  final int runTimeSeconds;
  final bool checkEngineLight;
  final TpmsData tpms;

  const CarTelemetry({
    this.speed = 0.0,
    this.rpm = 0.0,
    this.coolantTemp = 85.0,
    this.fuelLevel = 75.0,
    this.batteryVoltage = 12.6,
    this.engineLoad = 18.0,
    this.throttlePos = 0.0,
    this.intakeTemp = 30.0,
    this.ambientTemp = 28.0,
    this.odometer = 12450.0,
    this.runTimeSeconds = 0,
    this.checkEngineLight = false,
    this.tpms = const TpmsData(),
  });

  factory CarTelemetry.fromJson(Map<String, dynamic> json) {
    return CarTelemetry(
      speed: (json['speed'] as num?)?.toDouble() ?? 0.0,
      rpm: (json['rpm'] as num?)?.toDouble() ?? 0.0,
      coolantTemp: (json['coolant_temp'] as num?)?.toDouble() ??
          (json['coolantTemp'] as num?)?.toDouble() ??
          85.0,
      fuelLevel: (json['fuel_level'] as num?)?.toDouble() ??
          (json['fuelLevel'] as num?)?.toDouble() ??
          75.0,
      batteryVoltage: (json['battery_voltage'] as num?)?.toDouble() ??
          (json['batteryVoltage'] as num?)?.toDouble() ??
          12.6,
      engineLoad: (json['engine_load'] as num?)?.toDouble() ??
          (json['engineLoad'] as num?)?.toDouble() ??
          18.0,
      throttlePos: (json['throttle_pos'] as num?)?.toDouble() ??
          (json['throttlePos'] as num?)?.toDouble() ??
          0.0,
      intakeTemp: (json['intake_temp'] as num?)?.toDouble() ??
          (json['intakeTemp'] as num?)?.toDouble() ??
          30.0,
      ambientTemp: (json['ambient_temp'] as num?)?.toDouble() ??
          (json['ambientTemp'] as num?)?.toDouble() ??
          28.0,
      odometer: (json['odometer'] as num?)?.toDouble() ?? 12450.0,
      runTimeSeconds: (json['run_time'] as num?)?.toInt() ??
          (json['runTimeSeconds'] as num?)?.toInt() ??
          0,
      checkEngineLight: (json['check_engine'] as bool?) ?? false,
      tpms: json['tpms'] is Map<String, dynamic>
          ? TpmsData.fromJson(json['tpms'] as Map<String, dynamic>)
          : const TpmsData(),
    );
  }

  CarTelemetry copyWith({
    double? speed,
    double? rpm,
    double? coolantTemp,
    double? fuelLevel,
    double? batteryVoltage,
    double? engineLoad,
    double? throttlePos,
    double? intakeTemp,
    double? ambientTemp,
    double? odometer,
    int? runTimeSeconds,
    bool? checkEngineLight,
    TpmsData? tpms,
  }) {
    return CarTelemetry(
      speed: speed ?? this.speed,
      rpm: rpm ?? this.rpm,
      coolantTemp: coolantTemp ?? this.coolantTemp,
      fuelLevel: fuelLevel ?? this.fuelLevel,
      batteryVoltage: batteryVoltage ?? this.batteryVoltage,
      engineLoad: engineLoad ?? this.engineLoad,
      throttlePos: throttlePos ?? this.throttlePos,
      intakeTemp: intakeTemp ?? this.intakeTemp,
      ambientTemp: ambientTemp ?? this.ambientTemp,
      odometer: odometer ?? this.odometer,
      runTimeSeconds: runTimeSeconds ?? this.runTimeSeconds,
      checkEngineLight: checkEngineLight ?? this.checkEngineLight,
      tpms: tpms ?? this.tpms,
    );
  }
}
