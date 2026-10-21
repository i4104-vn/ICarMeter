import 'package:flutter/material.dart';
import '../../../models/telemetry.dart';

/// Helper for drawing the luxury vector car chassis and rubber wheels.
class CarChassisPainterHelper {
  static void drawWheels({
    required Canvas canvas,
    required TpmsData tpms,
    required double scale,
    required double flX,
    required double frX,
    required double frontY,
    required double rearY,
    required double wheelW,
    required double wheelH,
    required double wheelRadius,
    required double pressureAlert,
    required double tempAlert,
    required String? selectedTire,
  }) {
    final wheels = [
      ('fl', flX, frontY, tpms.fl),
      ('fr', frX, frontY, tpms.fr),
      ('rl', flX, rearY, tpms.rl),
      ('rr', frX, rearY, tpms.rr),
    ];

    for (final (posKey, wx, wy, tire) in wheels) {
      final isDisconnected = tire.isDisconnected;
      final isLow = tire.isLow(pressureAlert);
      final isHighTemp = tire.isHighTemp(tempAlert);
      final isAlert = isDisconnected || isLow || isHighTemp;
      final isSelected = selectedTire == posKey;

      final rect = Rect.fromLTWH(wx, wy, wheelW, wheelH);
      final rrect = RRect.fromRectAndRadius(rect, Radius.circular(wheelRadius));

      if (isAlert) {
        // Warning Red Glow
        final glowPaint = Paint()
          ..color = const Color(0x3DF53737)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 4.0 * scale
          ..strokeCap = StrokeCap.round;
        canvas.drawRRect(rrect, glowPaint);

        // Warning Red Translucent Fill
        final fillPaint = Paint()
          ..color = const Color(0x28F53737)
          ..style = PaintingStyle.fill;
        canvas.drawRRect(rrect, fillPaint);

        // Alert Red Border Stroke
        final strokePaint = Paint()
          ..color = const Color(0xFFF53737)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.6 * scale
          ..strokeCap = StrokeCap.round;
        canvas.drawRRect(rrect, strokePaint);

        // Center Indicator Line
        final rimPaint = Paint()
          ..color = const Color(0xDCFFFFFF)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.4 * scale
          ..strokeCap = StrokeCap.round;
        canvas.drawLine(
          Offset(wx + wheelW * 0.5, wy + 5.0 * scale),
          Offset(wx + wheelW * 0.5, wy + wheelH - 5.0 * scale),
          rimPaint,
        );
      } else {
        // Normal Dark Rubber Fill
        final fillPaint = Paint()
          ..color = const Color(0xFF161618)
          ..style = PaintingStyle.fill;
        canvas.drawRRect(rrect, fillPaint);

        // Subtle Border Stroke
        final strokePaint = Paint()
          ..color = const Color(0xFF4B4B50)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.3 * scale
          ..strokeCap = StrokeCap.round;
        canvas.drawRRect(rrect, strokePaint);

        // Wheel Rim Line
        final rimPaint = Paint()
          ..color = const Color(0xFF4B4B50)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.0 * scale
          ..strokeCap = StrokeCap.round;
        canvas.drawLine(
          Offset(wx + wheelW * 0.5, wy + 6.0 * scale),
          Offset(wx + wheelW * 0.5, wy + wheelH - 6.0 * scale),
          rimPaint,
        );
      }

      // Highlight selected wheel with cyan halo
      if (isSelected) {
        final selectPaint = Paint()
          ..color = const Color(0xFF00E5FF)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 2.0 * scale
          ..strokeCap = StrokeCap.round;
        canvas.drawRRect(
          RRect.fromRectAndRadius(
            rect.inflate(2.5 * scale),
            Radius.circular(wheelRadius + 2.0 * scale),
          ),
          selectPaint,
        );
      }
    }
  }

  static void drawCarChassis({
    required Canvas canvas,
    required double cx,
    required double cy,
    required double bodyW,
    required double bodyH,
    required double scale,
  }) {
    final bodyX = cx - bodyW / 2.0;
    final bodyY = cy - bodyH / 2.0;
    final rFront = 21.0 * scale;
    final rRear = 14.0 * scale;

    // Body contour path
    final bodyPath = Path()
      ..moveTo(bodyX + rFront, bodyY)
      ..lineTo(bodyX + bodyW - rFront, bodyY)
      ..quadraticBezierTo(bodyX + bodyW, bodyY, bodyX + bodyW, bodyY + rFront)
      ..lineTo(bodyX + bodyW, bodyY + bodyH - rRear)
      ..quadraticBezierTo(
          bodyX + bodyW, bodyY + bodyH, bodyX + bodyW - rRear, bodyY + bodyH)
      ..lineTo(bodyX + rRear, bodyY + bodyH)
      ..quadraticBezierTo(bodyX, bodyY + bodyH, bodyX, bodyY + bodyH - rRear)
      ..lineTo(bodyX, bodyY + rFront)
      ..quadraticBezierTo(bodyX, bodyY, bodyX + rFront, bodyY)
      ..close();

    // Body Fill (Deep luxury metallic obsidian black)
    final bodyFill = Paint()
      ..color = const Color(0xFF121214)
      ..style = PaintingStyle.fill;
    canvas.drawPath(bodyPath, bodyFill);

    // Body Border Stroke
    final bodyStroke = Paint()
      ..color = const Color(0xFF46464C)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.8 * scale
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round;
    canvas.drawPath(bodyPath, bodyStroke);

    // Front Windshield (Tinted glass arc)
    final fwPath = Path()
      ..moveTo(bodyX + 10.0 * scale, cy - 22.0 * scale)
      ..quadraticBezierTo(
        cx,
        cy - 34.0 * scale,
        bodyX + bodyW - 10.0 * scale,
        cy - 22.0 * scale,
      );
    final glassPaint = Paint()
      ..color = const Color(0xB48C9196)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.8 * scale
      ..strokeCap = StrokeCap.round;
    canvas.drawPath(fwPath, glassPaint);

    // Panoramic Sunroof outline
    final roofW = bodyW - 24.0 * scale;
    final roofH = 52.0 * scale;
    final roofRect = Rect.fromLTWH(
      cx - roofW / 2.0,
      cy - 18.0 * scale,
      roofW,
      roofH,
    );
    final roofPaint = Paint()
      ..color = const Color(0x8C2A2A2E)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.1 * scale;
    canvas.drawRRect(
      RRect.fromRectAndRadius(roofRect, Radius.circular(7.0 * scale)),
      roofPaint,
    );

    // Rear Windshield Arc
    final rwPath = Path()
      ..moveTo(bodyX + 11.0 * scale, cy + 39.0 * scale)
      ..quadraticBezierTo(
        cx,
        cy + 32.0 * scale,
        bodyX + bodyW - 11.0 * scale,
        cy + 39.0 * scale,
      );
    final rearGlassPaint = Paint()
      ..color = const Color(0xA08C9196)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5 * scale
      ..strokeCap = StrokeCap.round;
    canvas.drawPath(rwPath, rearGlassPaint);

    // Front Headlights (Twin sleek LED lightbars: Neon Cyan)
    final ledPaint = Paint()
      ..color = const Color(0xFFA5F0FF)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2.2 * scale
      ..strokeCap = StrokeCap.round;
    canvas.drawLine(
      Offset(bodyX + 8.0 * scale, bodyY + 11.0 * scale),
      Offset(bodyX + 21.0 * scale, bodyY + 5.0 * scale),
      ledPaint,
    );
    canvas.drawLine(
      Offset(bodyX + bodyW - 8.0 * scale, bodyY + 11.0 * scale),
      Offset(bodyX + bodyW - 21.0 * scale, bodyY + 5.0 * scale),
      ledPaint,
    );

    // Rear Taillights (Sleek red LED accents)
    final tailPaint = Paint()
      ..color = const Color(0xF0F02D2D)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2.0 * scale
      ..strokeCap = StrokeCap.round;
    canvas.drawLine(
      Offset(bodyX + 10.0 * scale, bodyY + bodyH - 5.0 * scale),
      Offset(bodyX + 22.0 * scale, bodyY + bodyH - 3.5 * scale),
      tailPaint,
    );
    canvas.drawLine(
      Offset(bodyX + bodyW - 10.0 * scale, bodyY + bodyH - 5.0 * scale),
      Offset(bodyX + bodyW - 22.0 * scale, bodyY + bodyH - 3.5 * scale),
      tailPaint,
    );
  }
}
