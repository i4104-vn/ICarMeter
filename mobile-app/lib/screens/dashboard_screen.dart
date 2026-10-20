import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../services/app_state.dart';
import '../widgets/speedometer_gauge.dart';

class DashboardScreen extends StatelessWidget {
  final AppState appState;
  final VoidCallback onNavigateToBt;
  final VoidCallback onNavigateToWifi;
  final VoidCallback onNavigateToTpms;

  const DashboardScreen({
    super.key,
    required this.appState,
    required this.onNavigateToBt,
    required this.onNavigateToWifi,
    required this.onNavigateToTpms,
  });

  Widget _buildTelemetryTile({
    required String label,
    required String value,
    required IconData icon,
    Color color = AppColors.primary,
  }) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
      decoration: BoxDecoration(
        color: AppColors.surface,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(color: AppColors.border, width: 1),
      ),
      child: Row(
        children: [
          Container(
            padding: const EdgeInsets.all(8),
            decoration: BoxDecoration(
              color: color.withOpacity(0.12),
              borderRadius: BorderRadius.circular(10),
            ),
            child: Icon(icon, color: color, size: 20),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  label,
                  style: const TextStyle(
                    fontSize: 11,
                    color: AppColors.textSecondary,
                  ),
                ),
                const SizedBox(height: 2),
                Text(
                  value,
                  style: const TextStyle(
                    fontSize: 16,
                    fontWeight: FontWeight.bold,
                    color: AppColors.text,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final t = appState.telemetry;
    final isConnected = appState.isBtConnected;

    return SingleChildScrollView(
      physics: const BouncingScrollPhysics(),
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Banner if not connected to Bluetooth
          if (!isConnected)
            Container(
              margin: const EdgeInsets.only(bottom: 16),
              padding: const EdgeInsets.all(14),
              decoration: BoxDecoration(
                gradient: LinearGradient(
                  colors: [
                    AppColors.warning.withOpacity(0.18),
                    AppColors.surface,
                  ],
                ),
                borderRadius: BorderRadius.circular(16),
                border: Border.all(color: AppColors.warning.withOpacity(0.6)),
              ),
              child: Row(
                children: [
                  const Icon(Icons.bluetooth_searching, color: AppColors.warning, size: 28),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: const [
                        Text(
                          'Chưa kết nối Bluetooth tới xe',
                          style: TextStyle(
                            fontSize: 13,
                            fontWeight: FontWeight.bold,
                            color: AppColors.text,
                          ),
                        ),
                        SizedBox(height: 2),
                        Text(
                          'Nếu chưa ghép nối, hãy kết nối Wi-Fi của xe để ghép nối Bluetooth trước.',
                          style: TextStyle(fontSize: 11, color: AppColors.textSecondary),
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(width: 8),
                  ElevatedButton(
                    onPressed: onNavigateToBt,
                    style: ElevatedButton.styleFrom(
                      backgroundColor: AppColors.primary,
                      foregroundColor: Colors.black,
                      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                      minimumSize: const Size(0, 36),
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(10),
                      ),
                    ),
                    child: const Text('Ghép nối', style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold)),
                  ),
                ],
              ),
            ),

          // Speedometer
          SpeedometerGauge(
            speed: t.speed,
            unit: appState.storage.speedUnit.toUpperCase(),
            checkEngine: t.checkEngineLight,
            highTemp: t.coolantTemp > 105,
            lowBattery: t.batteryVoltage < 11.8,
          ),

          const SizedBox(height: 16),

          // Active Trip Banner
          if (appState.activeTrip != null)
            Container(
              margin: const EdgeInsets.only(bottom: 16),
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
              decoration: BoxDecoration(
                color: AppColors.surface,
                borderRadius: BorderRadius.circular(14),
                border: Border.all(color: AppColors.primary.withOpacity(0.5)),
              ),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Row(
                    children: [
                      const Icon(Icons.play_circle_fill, color: AppColors.primary, size: 20),
                      const SizedBox(width: 8),
                      Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          const Text(
                            'Chuyến đi hiện tại',
                            style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold, color: AppColors.text),
                          ),
                          Text(
                            '${(appState.activeTrip!.durationSeconds / 60).toStringAsFixed(0)} phút • TB: ${appState.activeTrip!.avgSpeedKmh.toStringAsFixed(1)} km/h',
                            style: const TextStyle(fontSize: 11, color: AppColors.textSecondary),
                          ),
                        ],
                      ),
                    ],
                  ),
                  Text(
                    '${appState.activeTrip!.distanceKm.toStringAsFixed(1)} km',
                    style: const TextStyle(
                      fontSize: 16,
                      fontWeight: FontWeight.bold,
                      color: AppColors.primary,
                    ),
                  ),
                ],
              ),
            ),

          // Telemetry Cards Grid
          GridView.count(
            crossAxisCount: 2,
            shrinkWrap: true,
            physics: const NeverScrollableScrollPhysics(),
            mainAxisSpacing: 10,
            crossAxisSpacing: 10,
            childAspectRatio: 2.2,
            children: [
              _buildTelemetryTile(
                label: 'Nhiệt độ nước làm mát',
                value: '${t.coolantTemp.toStringAsFixed(0)}°C',
                icon: Icons.thermostat,
                color: t.coolantTemp > 100 ? AppColors.danger : AppColors.primary,
              ),
              _buildTelemetryTile(
                label: 'Điện áp ắc quy',
                value: '${t.batteryVoltage.toStringAsFixed(1)} V',
                icon: Icons.electric_bolt,
                color: t.batteryVoltage < 12.0 ? AppColors.warning : AppColors.success,
              ),
              _buildTelemetryTile(
                label: 'Tải động cơ',
                value: '${t.engineLoad.toStringAsFixed(0)}%',
                icon: Icons.speed,
                color: AppColors.info,
              ),
              _buildTelemetryTile(
                label: 'Mức nhiên liệu',
                value: '${t.fuelLevel.toStringAsFixed(0)}%',
                icon: Icons.local_gas_station,
                color: t.fuelLevel < 15 ? AppColors.danger : AppColors.primary,
              ),
              _buildTelemetryTile(
                label: 'Bướm ga',
                value: '${t.throttlePos.toStringAsFixed(0)}%',
                icon: Icons.tune,
                color: AppColors.secondary,
              ),
              _buildTelemetryTile(
                label: 'Nhiệt độ khí nạp',
                value: '${t.intakeTemp.toStringAsFixed(0)}°C',
                icon: Icons.air,
                color: AppColors.textSecondary,
              ),
            ],
          ),

          const SizedBox(height: 16),

          // TPMS Quick Overview Card
          GestureDetector(
            onTap: onNavigateToTpms,
            child: Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: AppColors.surface,
                borderRadius: BorderRadius.circular(16),
                border: Border.all(color: AppColors.border, width: 1),
              ),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Row(
                    children: [
                      Container(
                        padding: const EdgeInsets.all(10),
                        decoration: BoxDecoration(
                          color: AppColors.primary.withOpacity(0.12),
                          borderRadius: BorderRadius.circular(12),
                        ),
                        child: const Icon(Icons.tire_repair, color: AppColors.primary, size: 24),
                      ),
                      const SizedBox(width: 14),
                      Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          const Text(
                            'Áp suất lốp (TPMS)',
                            style: TextStyle(
                              fontSize: 14,
                              fontWeight: FontWeight.bold,
                              color: AppColors.text,
                            ),
                          ),
                          const SizedBox(height: 2),
                          Text(
                            'FL: ${t.tpms.fl.pressureBar.toStringAsFixed(1)} bar • FR: ${t.tpms.fr.pressureBar.toStringAsFixed(1)} bar',
                            style: const TextStyle(fontSize: 12, color: AppColors.textSecondary),
                          ),
                        ],
                      ),
                    ],
                  ),
                  const Icon(Icons.chevron_right, color: AppColors.textSecondary),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
