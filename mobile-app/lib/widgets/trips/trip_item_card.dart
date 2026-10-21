import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import '../../theme/app_theme.dart';
import '../../models/trip.dart';

/// Single item tile displaying date, distance, duration, and speed metrics for a trip.
class TripItemCard extends StatelessWidget {
  final TripRecord trip;

  const TripItemCard({super.key, required this.trip});

  @override
  Widget build(BuildContext context) {
    final dateFormat = DateFormat('HH:mm - dd/MM/yyyy');
    final durationMin = (trip.durationSeconds / 60).round();

    return Container(
      margin: const EdgeInsets.only(bottom: 10),
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: AppColors.surface,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(color: AppColors.border, width: 1),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Row(
                children: [
                  const Icon(Icons.calendar_today, size: 14, color: AppColors.primary),
                  const SizedBox(width: 6),
                  Text(
                    dateFormat.format(trip.startTime),
                    style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w600, color: AppColors.text),
                  ),
                ],
              ),
              Text(
                '${trip.distanceKm.toStringAsFixed(1)} km',
                style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold, color: AppColors.primary),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                'Thời gian: $durationMin phút',
                style: const TextStyle(fontSize: 11, color: AppColors.textSecondary),
              ),
              Text(
                'TB: ${trip.avgSpeedKmh.toStringAsFixed(1)} km/h  •  Max: ${trip.maxSpeedKmh.toStringAsFixed(0)} km/h',
                style: const TextStyle(fontSize: 11, color: AppColors.textMuted),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
