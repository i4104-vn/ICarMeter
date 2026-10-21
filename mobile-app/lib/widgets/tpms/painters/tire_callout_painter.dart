import 'package:flutter/material.dart';
import '../../../models/telemetry.dart';

/// Helper for drawing tire callouts, leader lines, alert glowing boxes, and readings.
class TireCalloutPainterHelper {
  static void drawTpmsOverlay({
    required Canvas canvas,
    required Size size,
    required TpmsData tpms,
    required double cx,
    required double cy,
    required double scale,
    required double flX,
    required double frX,
    required double frontY,
    required double rearY,
    required double wheelW,
    required double wheelH,
    required double pressureAlert,
    required double tempAlert,
    required String pressureUnit,
    required String tempUnit,
  }) {
    final leftTargetX = flX - 4.0 * scale;
    final rightTargetX = frX + wheelW + 4.0 * scale;

    final targetFl = Offset(leftTargetX, frontY + wheelH / 2.0);
    final targetFr = Offset(rightTargetX, frontY + wheelH / 2.0);
    final targetRl = Offset(leftTargetX, rearY + wheelH / 2.0);
    final targetRr = Offset(rightTargetX, rearY + wheelH / 2.0);

    final leftTextX = 18.0 * scale;
    final rightTextX = size.width - 18.0 * scale;
    final leftMidX = cx - 68.0 * scale;
    final rightMidX = cx + 68.0 * scale;

    final frontLineY = cy - 62.0 * scale;
    final rearLineY = cy + 62.0 * scale;

    final callouts = [
      (tpms.fl, targetFl, leftTextX, leftMidX, frontLineY, false),
      (tpms.fr, targetFr, rightTextX, rightMidX, frontLineY, true),
      (tpms.rl, targetRl, leftTextX, leftMidX, rearLineY, false),
      (tpms.rr, targetRr, rightTextX, rightMidX, rearLineY, true),
    ];

    for (final (tire, target, textX, lineMidX, lineY, alignRight) in callouts) {
      drawSingleTireCallout(
        canvas: canvas,
        scale: scale,
        tire: tire,
        target: target,
        textX: textX,
        lineMidX: lineMidX,
        lineY: lineY,
        alignRight: alignRight,
        pressureAlert: pressureAlert,
        tempAlert: tempAlert,
        pressureUnit: pressureUnit,
        tempUnit: tempUnit,
      );
    }
  }

  static void drawSingleTireCallout({
    required Canvas canvas,
    required double scale,
    required TireData tire,
    required Offset target,
    required double textX,
    required double lineMidX,
    required double lineY,
    required bool alignRight,
    required double pressureAlert,
    required double tempAlert,
    required String pressureUnit,
    required String tempUnit,
  }) {
    final isDisconnected = tire.isDisconnected;
    final isLow = tire.isLow(pressureAlert);
    final isHighTemp = tire.isHighTemp(tempAlert);
    final isAlert = isDisconnected || isLow || isHighTemp;

    final alertColor = const Color(0xFFF53737);
    final normalLineColor = const Color(0xFF00E5FF);
    final lineColor = isAlert ? alertColor : normalLineColor;

    // Leader line with bend to wheel target
    final linePaint = Paint()
      ..color = lineColor
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.2 * scale
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round;

    final linePath = Path()
      ..moveTo(textX, lineY)
      ..lineTo(lineMidX, lineY)
      ..lineTo(target.dx, target.dy);
    canvas.drawPath(linePath, linePaint);

    // Target contact dot on wheel
    final dotPaint = Paint()
      ..color = lineColor
      ..style = PaintingStyle.fill;
    canvas.drawCircle(target, 2.2 * scale, dotPaint);

    if (isDisconnected) {
      // Disconnected: Render "▲ ERROR" badge + "-- °C"
      final badgeW = 74.0 * scale;
      final badgeH = 20.0 * scale;
      final badgeX = alignRight ? textX - badgeW : textX;
      final badgeY = lineY - 25.0 * scale;

      final badgeRect = Rect.fromLTWH(badgeX, badgeY, badgeW, badgeH);
      final badgeRRect =
          RRect.fromRectAndRadius(badgeRect, Radius.circular(4.0 * scale));

      final fillPaint = Paint()
        ..color = const Color(0x1AEB2323)
        ..style = PaintingStyle.fill;
      canvas.drawRRect(badgeRRect, fillPaint);

      final borderPaint = Paint()
        ..color = const Color(0x8CEB3737)
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.0 * scale;
      canvas.drawRRect(badgeRRect, borderPaint);

      final iconX = badgeX + 13.0 * scale;
      final iconY = badgeY + badgeH / 2.0;
      final triPath = Path()
        ..moveTo(iconX, iconY - 4.5 * scale)
        ..lineTo(iconX + 4.8 * scale, iconY + 4.0 * scale)
        ..lineTo(iconX - 4.8 * scale, iconY + 4.0 * scale)
        ..close();

      final triPaint = Paint()
        ..color = alertColor
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.2 * scale
        ..strokeCap = StrokeCap.round
        ..strokeJoin = StrokeJoin.round;
      canvas.drawPath(triPath, triPaint);

      canvas.drawLine(
        Offset(iconX, iconY - 2.0 * scale),
        Offset(iconX, iconY + 0.8 * scale),
        triPaint,
      );
      canvas.drawCircle(Offset(iconX, iconY + 2.7 * scale), 0.7 * scale, dotPaint);

      final errorTp = TextPainter(
        text: TextSpan(
          text: 'ERROR',
          style: TextStyle(
            color: alertColor,
            fontSize: 11.5 * scale,
            fontWeight: FontWeight.bold,
            letterSpacing: 0.5,
          ),
        ),
        textDirection: TextDirection.ltr,
      )..layout();
      errorTp.paint(canvas, Offset(badgeX + 23.0 * scale, badgeY + 3.0 * scale));

      final tempTp = TextPainter(
        text: TextSpan(
          text: '-- °C',
          style: TextStyle(
            color: const Color(0xDCA0A8B4),
            fontSize: 14.5 * scale,
            fontWeight: FontWeight.w600,
          ),
        ),
        textDirection: TextDirection.ltr,
      )..layout();
      final tempX = alignRight ? textX - tempTp.width : textX;
      tempTp.paint(canvas, Offset(tempX, lineY + 6.0 * scale));
    } else {
      // Connected: Render pressure value + unit + temperature
      final pColor = isLow ? alertColor : Colors.white;
      final pVal = pressureUnit == 'psi'
          ? (tire.pressureBar * 14.5038).toStringAsFixed(1)
          : tire.pressureBar.toStringAsFixed(1);
      final pUnit = pressureUnit == 'psi' ? ' PSI' : ' bar';

      final pressureTp = TextPainter(
        text: TextSpan(
          children: [
            TextSpan(
              text: pVal,
              style: TextStyle(
                color: pColor,
                fontSize: 20.0 * scale,
                fontWeight: FontWeight.bold,
              ),
            ),
            TextSpan(
              text: pUnit,
              style: TextStyle(
                color: isLow ? alertColor : const Color(0xFF94A3B8),
                fontSize: 12.0 * scale,
                fontWeight: FontWeight.normal,
              ),
            ),
          ],
        ),
        textDirection: TextDirection.ltr,
      )..layout();

      final pX = alignRight ? textX - pressureTp.width : textX;
      pressureTp.paint(canvas, Offset(pX, lineY - 26.0 * scale));

      final tColor = isHighTemp ? alertColor : const Color(0xFFDCE2EC);
      final tVal = tempUnit == 'F'
          ? '${((tire.tempC * 9 / 5) + 32).toStringAsFixed(0)}°F'
          : '${tire.tempC.toStringAsFixed(0)}°C';

      final tempTp = TextPainter(
        text: TextSpan(
          text: tVal,
          style: TextStyle(
            color: tColor,
            fontSize: 14.0 * scale,
            fontWeight: FontWeight.bold,
          ),
        ),
        textDirection: TextDirection.ltr,
      )..layout();

      final tX = alignRight ? textX - tempTp.width : textX;
      tempTp.paint(canvas, Offset(tX, lineY + 5.0 * scale));
    }
  }
}
