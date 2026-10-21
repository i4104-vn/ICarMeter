import 'package:flutter/material.dart';
import '../../theme/app_theme.dart';
import '../../models/trip.dart';

/// Card showing aggregate statistics for all recorded vehicle trips.
class TripSummaryCard extends StatelessWidget {
  final List<TripRecord> trips;

  const TripSummaryCard({super.key, required this.trips});

  Widget _buildStatColumn(String label, String value, IconData icon) {
    return Column(
      children: [
        Icon(icon, size: 20, color: AppColors.primary),
        const SizedBox(height: 6),
        Text(
          value,
          style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold, color: AppColors.text),
        ),
        const SizedBox(height: 2),
        Text(
          label,
          style: const TextStyle(fontSize: 11, color: AppColors.textSecondary),
        ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    double totalDistance = 0.0;
    double topSpeed = 0.0;
    for (final t in trips) {
      totalDistance += t.distanceKm;
      if (t.maxSpeedKmh > topSpeed) topSpeed = t.maxSpeedKmh;
    }

    return Container(
      padding: const EdgeInsets.all(18),
      decoration: BoxDecoration(
        gradient: AppColors.cardGradient,
        borderRadius: BorderRadius.circular(18),
        border: Border.all(color: AppColors.primary.withOpacity(0.3), width: 1.2),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceAround,
        children: [
          _buildStatColumn('Tổng quãng đường', '${totalDistance.toStringAsFixed(1)} km', Icons.alt_route),
          Container(width: 1, height: 40, color: AppColors.border),
          _buildStatColumn('Số chuyến đi', '${trips.length}', Icons.flag),
          Container(width: 1, height: 40, color: AppColors.border),
          _buildStatColumn('Tốc độ tối đa', '${topSpeed.toStringAsFixed(0)} km/h', Icons.speed),
        ],
      ),
    );
  }
}
