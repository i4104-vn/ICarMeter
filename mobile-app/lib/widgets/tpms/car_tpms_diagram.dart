import 'package:flutter/material.dart';
import '../../models/telemetry.dart';
import 'painters/car_chassis_painter.dart';
import 'painters/tire_callout_painter.dart';

/// Interactive vector 4-wheel TPMS diagram replicating the Pi dashboard.
class CarTpmsDiagram extends StatelessWidget {
  final TpmsData tpms;
  final double pressureAlert;
  final double tempAlert;
  final String pressureUnit;
  final String tempUnit;
  final String? selectedTire;
  final ValueChanged<String>? onTireTap;

  const CarTpmsDiagram({
    super.key,
    required this.tpms,
    this.pressureAlert = 2.0,
    this.tempAlert = 75.0,
    this.pressureUnit = 'bar',
    this.tempUnit = 'C',
    this.selectedTire,
    this.onTireTap,
  });

  void _handleTap(TapDownDetails details, Size size) {
    if (onTireTap == null) return;
    final isTop = details.localPosition.dy < size.height / 2;
    final isLeft = details.localPosition.dx < size.width / 2;

    final target = switch ((isTop, isLeft)) {
      (true, true) => 'fl',
      (true, false) => 'fr',
      (false, true) => 'rl',
      (false, false) => 'rr',
    };
    onTireTap!(target);
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      height: 290,
      decoration: BoxDecoration(
        color: const Color(0xFF0B0E14),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(
          color: const Color(0x3300E5FF),
          width: 1.2,
        ),
      ),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(20),
        child: LayoutBuilder(
          builder: (context, constraints) {
            final size = Size(constraints.maxWidth, constraints.maxHeight);
            return GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTapDown: (details) => _handleTap(details, size),
              child: CustomPaint(
                size: size,
                painter: _CarTpmsPainter(
                  tpms: tpms,
                  pressureAlert: pressureAlert,
                  tempAlert: tempAlert,
                  pressureUnit: pressureUnit,
                  tempUnit: tempUnit,
                  selectedTire: selectedTire,
                ),
              ),
            );
          },
        ),
      ),
    );
  }
}

class _CarTpmsPainter extends CustomPainter {
  final TpmsData tpms;
  final double pressureAlert;
  final double tempAlert;
  final String pressureUnit;
  final String tempUnit;
  final String? selectedTire;

  _CarTpmsPainter({
    required this.tpms,
    required this.pressureAlert,
    required this.tempAlert,
    required this.pressureUnit,
    required this.tempUnit,
    this.selectedTire,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final cx = size.width / 2.0;
    final cy = size.height / 2.0;
    final scale = (size.height / 300.0).clamp(0.75, 1.35);

    final bodyW = 78.0 * scale;
    final bodyH = 152.0 * scale;
    final wheelW = 13.0 * scale;
    final wheelH = 32.0 * scale;
    final wheelRadius = 3.5 * scale;
    final bodyHalfW = bodyW / 2.0;

    final flX = cx - bodyHalfW - wheelW - 3.0 * scale;
    final frX = cx + bodyHalfW + 3.0 * scale;
    final frontY = cy - 52.0 * scale;
    final rearY = cy + 20.0 * scale;

    // 1. Draw wheels & alerts
    CarChassisPainterHelper.drawWheels(
      canvas: canvas,
      tpms: tpms,
      scale: scale,
      flX: flX,
      frX: frX,
      frontY: frontY,
      rearY: rearY,
      wheelW: wheelW,
      wheelH: wheelH,
      wheelRadius: wheelRadius,
      pressureAlert: pressureAlert,
      tempAlert: tempAlert,
      selectedTire: selectedTire,
    );

    // 2. Draw Vector Car Chassis Body
    CarChassisPainterHelper.drawCarChassis(
      canvas: canvas,
      cx: cx,
      cy: cy,
      bodyW: bodyW,
      bodyH: bodyH,
      scale: scale,
    );

    // 3. Draw Tire Readings & Leader Lines
    TireCalloutPainterHelper.drawTpmsOverlay(
      canvas: canvas,
      size: size,
      tpms: tpms,
      cx: cx,
      cy: cy,
      scale: scale,
      flX: flX,
      frX: frX,
      frontY: frontY,
      rearY: rearY,
      wheelW: wheelW,
      wheelH: wheelH,
      pressureAlert: pressureAlert,
      tempAlert: tempAlert,
      pressureUnit: pressureUnit,
      tempUnit: tempUnit,
    );
  }

  @override
  bool shouldRepaint(covariant _CarTpmsPainter old) {
    return old.tpms != tpms ||
        old.pressureAlert != pressureAlert ||
        old.tempAlert != tempAlert ||
        old.pressureUnit != pressureUnit ||
        old.tempUnit != tempUnit ||
        old.selectedTire != selectedTire;
  }
}
