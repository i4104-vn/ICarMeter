import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import '../theme/app_theme.dart';
import '../services/app_state.dart';
import '../models/trip.dart';

class TripsScreen extends StatelessWidget {
  final AppState appState;

  const TripsScreen({super.key, required this.appState});

  Widget _buildSummaryCard(List<TripRecord> trips) {
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

  Widget _buildTripTile(TripRecord trip) {
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

  @override
  Widget build(BuildContext context) {
    final trips = appState.trips;

    return Scaffold(
      backgroundColor: Colors.transparent,
      body: SingleChildScrollView(
        physics: const BouncingScrollPhysics(),
        padding: const EdgeInsets.fromLTRB(16, 12, 16, 24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            _buildSummaryCard(trips),
            const SizedBox(height: 20),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                const Text(
                  'Lịch sử chuyến đi',
                  style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold, color: AppColors.text),
                ),
                if (trips.isNotEmpty)
                  TextButton.icon(
                    onPressed: () => appState.clearTrips(),
                    icon: const Icon(Icons.delete_outline, size: 16, color: AppColors.textMuted),
                    label: const Text('Xóa lịch sử', style: TextStyle(fontSize: 12, color: AppColors.textMuted)),
                  ),
              ],
            ),
            const SizedBox(height: 10),
            if (trips.isEmpty)
              Container(
                padding: const EdgeInsets.symmetric(vertical: 40),
                alignment: Alignment.center,
                child: Column(
                  children: const [
                    Icon(Icons.route, size: 48, color: AppColors.textMuted),
                    SizedBox(height: 12),
                    Text(
                      'Chưa có dữ liệu chuyến đi',
                      style: TextStyle(fontSize: 14, color: AppColors.textSecondary),
                    ),
                    SizedBox(height: 4),
                    Text(
                      'Hệ thống sẽ tự động ghi lại khi xe di chuyển trên 5 km/h.',
                      style: TextStyle(fontSize: 11, color: AppColors.textMuted),
                    ),
                  ],
                ),
              )
            else
              ...trips.map(_buildTripTile),
          ],
        ),
      ),
    );
  }
}
