import 'dart:math' as math;
import 'package:flutter/material.dart';
import '../../theme/app_theme.dart';

/// Circular modern digital speedometer gauge with active gradient arc and warning icons.
class SpeedometerGauge extends StatelessWidget {
  final double speed;
  final double maxSpeed;
  final double rpm;
  final String unit;
  final bool checkEngine;
  final bool highTemp;
  final bool lowBattery;

  const SpeedometerGauge({
    super.key,
    required this.speed,
    this.maxSpeed = 240,
    this.rpm = 0,
    this.unit = 'KM/H',
    this.checkEngine = false,
    this.highTemp = false,
    this.lowBattery = false,
  });

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final size = math.min(constraints.maxWidth, 280.0);

        return Center(
          child: SizedBox(
            width: size,
            height: size * 0.9,
            child: Stack(
              alignment: Alignment.center,
              children: [
                CustomPaint(
                  size: Size(size, size),
                  painter: _SpeedometerPainter(
                    speed: speed,
                    maxSpeed: maxSpeed,
                    rpm: rpm,
                  ),
                ),
                Positioned(
                  top: size * 0.35,
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(
                        speed.toInt().toString(),
                        style: TextStyle(
                          fontSize: size * 0.22,
                          fontWeight: FontWeight.bold,
                          color: AppColors.text,
                          letterSpacing: -1,
                          height: 1.0,
                          shadows: [
                            Shadow(
                              color: AppColors.primary.withOpacity(0.5),
                              blurRadius: 16,
                            ),
                          ],
                        ),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        unit,
                        style: const TextStyle(
                          fontSize: 12,
                          fontWeight: FontWeight.bold,
                          letterSpacing: 2,
                          color: AppColors.primary,
                        ),
                      ),
                      const SizedBox(height: 10),
                      Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          if (checkEngine)
                            const Padding(
                              padding: EdgeInsets.symmetric(horizontal: 4),
                              child: Icon(Icons.warning_amber, color: AppColors.danger, size: 20),
                            ),
                          if (highTemp)
                            const Padding(
                              padding: EdgeInsets.symmetric(horizontal: 4),
                              child: Icon(Icons.thermostat, color: AppColors.danger, size: 20),
                            ),
                          if (lowBattery)
                            const Padding(
                              padding: EdgeInsets.symmetric(horizontal: 4),
                              child: Icon(Icons.battery_alert, color: AppColors.warning, size: 20),
                            ),
                        ],
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}

class _SpeedometerPainter extends CustomPainter {
  final double speed;
  final double maxSpeed;
  final double rpm;

  _SpeedometerPainter({
    required this.speed,
    required this.maxSpeed,
    required this.rpm,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final center = Offset(size.width / 2, size.height / 2);
    final radius = (size.width / 2) - 16;

    const startAngle = 135 * (math.pi / 180);
    const sweepAngle = 270 * (math.pi / 180);

    // Background track
    final trackPaint = Paint()
      ..color = AppColors.surfaceLight.withOpacity(0.5)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 14
      ..strokeCap = StrokeCap.round;

    canvas.drawArc(
      Rect.fromCircle(center: center, radius: radius),
      startAngle,
      sweepAngle,
      false,
      trackPaint,
    );

    // Active speed arc
    final speedRatio = (speed / maxSpeed).clamp(0.0, 1.0);
    final activeSweep = sweepAngle * speedRatio;

    if (activeSweep > 0) {
      final activePaint = Paint()
        ..shader = const SweepGradient(
          colors: [AppColors.secondary, AppColors.primary],
          startAngle: startAngle,
          endAngle: startAngle + sweepAngle,
        ).createShader(Rect.fromCircle(center: center, radius: radius))
        ..style = PaintingStyle.stroke
        ..strokeWidth = 14
        ..strokeCap = StrokeCap.round;

      canvas.drawArc(
        Rect.fromCircle(center: center, radius: radius),
        startAngle,
        activeSweep,
        false,
        activePaint,
      );
    }

    // Ticks
    final tickPaint = Paint()
      ..color = AppColors.border
      ..strokeWidth = 2;

    for (int i = 0; i <= 24; i++) {
      final angle = startAngle + (sweepAngle * (i / 24));
      final isMajor = i % 4 == 0;
      final innerRadius = radius - (isMajor ? 20 : 12);
      final outerRadius = radius - 8;

      final p1 = Offset(
        center.dx + innerRadius * math.cos(angle),
        center.dy + innerRadius * math.sin(angle),
      );
      final p2 = Offset(
        center.dx + outerRadius * math.cos(angle),
        center.dy + outerRadius * math.sin(angle),
      );

      tickPaint.color = (i / 24) <= speedRatio ? AppColors.primary : AppColors.border;
      canvas.drawLine(p1, p2, tickPaint);
    }
  }

  @override
  bool shouldRepaint(covariant _SpeedometerPainter oldDelegate) {
    return oldDelegate.speed != speed ||
        oldDelegate.maxSpeed != maxSpeed ||
        oldDelegate.rpm != rpm;
  }
}
