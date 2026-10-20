import 'package:flutter/material.dart';
import '../services/app_state.dart';
import '../widgets/speedometer_gauge.dart';

class DashboardScreen extends StatelessWidget {
  final AppState appState;

  const DashboardScreen({
    super.key,
    required this.appState,
  });

  @override
  Widget build(BuildContext context) {
    final t = appState.telemetry;

    return Center(
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
        child: FittedBox(
          fit: BoxFit.contain,
          child: SpeedometerGauge(
            speed: t.speed,
            unit: appState.storage.speedUnit.toUpperCase(),
            maxSpeed: 240,
          ),
        ),
      ),
    );
  }
}
