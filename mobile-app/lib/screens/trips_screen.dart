import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../services/app_state.dart';
import '../widgets/trips/trip_summary_card.dart';
import '../widgets/trips/trip_item_card.dart';

/// Screen displaying historical vehicle journeys and aggregated trip metrics.
class TripsScreen extends StatelessWidget {
  final AppState appState;

  const TripsScreen({super.key, required this.appState});

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
            TripSummaryCard(trips: trips),
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
                child: const Column(
                  children: [
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
              ...trips.map((t) => TripItemCard(trip: t)),
          ],
        ),
      ),
    );
  }
}
