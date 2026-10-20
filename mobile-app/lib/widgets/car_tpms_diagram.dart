import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../models/telemetry.dart';

class CarTpmsDiagram extends StatelessWidget {
  final TpmsData tpms;
  final double pressureAlert;
  final double tempAlert;
  final String pressureUnit;
  final String tempUnit;
  final void Function(String wheelPosition)? onTireTap;

  const CarTpmsDiagram({
    super.key,
    required this.tpms,
    this.pressureAlert = 1.8,
    this.tempAlert = 70.0,
    this.pressureUnit = 'bar',
    this.tempUnit = 'C',
    this.onTireTap,
  });

  Color _getTireColor(TireData tire) {
    if (tire.pressureBar < 1.4 || tire.pressureBar > 3.0 || tire.tempC >= tempAlert) {
      return AppColors.danger;
    }
    if (tire.pressureBar < pressureAlert) {
      return AppColors.warning;
    }
    return AppColors.primary;
  }

  String _formatPressure(double pBar) {
    if (pressureUnit == 'psi') {
      return '${(pBar * 14.5038).toStringAsFixed(1)} PSI';
    }
    return '${pBar.toStringAsFixed(2)} bar';
  }

  String _formatTemp(double tC) {
    if (tempUnit == 'F') {
      return '${((tC * 9 / 5) + 32).toStringAsFixed(0)}°F';
    }
    return '${tC.toStringAsFixed(0)}°C';
  }

  Widget _buildTireCard({
    required String label,
    required TireData tire,
    required String positionKey,
  }) {
    final color = _getTireColor(tire);

    return GestureDetector(
      onTap: () => onTireTap?.call(positionKey),
      child: Container(
        width: 130,
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
        decoration: BoxDecoration(
          color: AppColors.surface,
          borderRadius: BorderRadius.circular(12),
          border: Border.all(color: color.withOpacity(0.7), width: 1.5),
          boxShadow: [
            BoxShadow(
              color: color.withOpacity(0.15),
              blurRadius: 8,
              offset: const Offset(0, 2),
            ),
          ],
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  label,
                  style: TextStyle(
                    fontSize: 11,
                    fontWeight: FontWeight.bold,
                    color: color,
                  ),
                ),
                Icon(Icons.tire_repair, size: 14, color: color),
              ],
            ),
            const SizedBox(height: 4),
            Text(
              _formatPressure(tire.pressureBar),
              style: const TextStyle(
                fontSize: 15,
                fontWeight: FontWeight.bold,
                color: AppColors.text,
              ),
            ),
            const SizedBox(height: 2),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  _formatTemp(tire.tempC),
                  style: const TextStyle(fontSize: 11, color: AppColors.textSecondary),
                ),
                Text(
                  '${tire.batteryPercent}%',
                  style: const TextStyle(fontSize: 10, color: AppColors.textMuted),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: AppColors.surfaceLight.withOpacity(0.3),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(color: AppColors.border, width: 1),
      ),
      child: Column(
        children: [
          // Front Axle
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              _buildTireCard(label: 'TRƯỚC TRÁI (FL)', tire: tpms.fl, positionKey: 'fl'),
              _buildTireCard(label: 'TRƯỚC PHẢI (FR)', tire: tpms.fr, positionKey: 'fr'),
            ],
          ),
          const SizedBox(height: 14),

          // Car Silhouette Icon & Direction Indicator
          Center(
            child: Stack(
              alignment: Alignment.center,
              children: [
                Container(
                  width: 80,
                  height: 120,
                  decoration: BoxDecoration(
                    color: AppColors.surface.withOpacity(0.8),
                    borderRadius: BorderRadius.circular(28),
                    border: Border.all(color: AppColors.border, width: 1.5),
                  ),
                ),
                Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(Icons.arrow_upward, size: 20, color: AppColors.primary.withOpacity(0.7)),
                    const SizedBox(height: 6),
                    Icon(Icons.directions_car, size: 48, color: AppColors.textSecondary.withOpacity(0.5)),
                  ],
                ),
              ],
            ),
          ),
          const SizedBox(height: 14),

          // Rear Axle
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              _buildTireCard(label: 'SAU TRÁI (RL)', tire: tpms.rl, positionKey: 'rl'),
              _buildTireCard(label: 'SAU PHẢI (RR)', tire: tpms.rr, positionKey: 'rr'),
            ],
          ),
        ],
      ),
    );
  }
}
