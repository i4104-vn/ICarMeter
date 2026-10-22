import 'package:flutter/material.dart';
import '../../theme/app_theme.dart';
import '../../services/app_state.dart';
import 'bt_logs_tab.dart';

void showBluetoothModal(BuildContext context, AppState appState) {
  showModalBottomSheet(
    context: context,
    isScrollControlled: true,
    backgroundColor: Colors.transparent,
    builder: (ctx) => BtPairingModal(appState: appState),
  );
}

/// Modal bottom sheet for Bluetooth management: direct status and live diagnostic logs without tabs.
class BtPairingModal extends StatefulWidget {
  final AppState appState;

  const BtPairingModal({
    super.key,
    required this.appState,
  });

  @override
  State<BtPairingModal> createState() => _BtPairingModalState();
}

class _BtPairingModalState extends State<BtPairingModal> {
  @override
  void initState() {
    super.initState();
    if (widget.appState.isBtConnected) {
      _requestBluetoothLogs();
    }
  }

  void _requestBluetoothLogs() {
    final cmd = {'type': 'get_bt_logs'};
    if (widget.appState.isBtConnected) {
      widget.appState.sendBtCommand(cmd);
    }
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.appState,
      builder: (context, _) {
        final state = widget.appState;
        final isConnected = state.isBtConnected;
        final isConnecting = state.isBtConnecting;
        final piMac = state.btCoordinator.piMac;
        final piName = state.btCoordinator.piName.isNotEmpty
            ? state.btCoordinator.piName
            : 'ICarMeter';
        final connectedName = state.connectedName.isNotEmpty
            ? state.connectedName
            : (piName.isNotEmpty ? piName : 'ICarMeter');
        final connectedMac = state.connectedMac.isNotEmpty
            ? state.connectedMac
            : (piMac.isNotEmpty ? piMac : 'Chưa có MAC');

        return Container(
          height: MediaQuery.of(context).size.height * 0.90,
          decoration: const BoxDecoration(
            color: AppColors.background,
            borderRadius: BorderRadius.vertical(top: Radius.circular(24)),
          ),
          child: Column(
            children: [
              // 1. Header with Drag Handle and Close button
              Container(
                padding: const EdgeInsets.fromLTRB(16, 10, 12, 10),
                decoration: const BoxDecoration(
                  color: AppColors.surface,
                  borderRadius: BorderRadius.vertical(top: Radius.circular(24)),
                  border: Border(bottom: BorderSide(color: AppColors.border)),
                ),
                child: Column(
                  children: [
                    Container(
                      width: 40,
                      height: 4,
                      decoration: BoxDecoration(
                        color: AppColors.border,
                        borderRadius: BorderRadius.circular(2),
                      ),
                    ),
                    const SizedBox(height: 10),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        const Row(
                          children: [
                            Icon(Icons.bluetooth, color: AppColors.primary, size: 22),
                            SizedBox(width: 8),
                            Text(
                              'Bluetooth ICarMeter',
                              style: TextStyle(
                                fontSize: 16,
                                fontWeight: FontWeight.bold,
                                color: AppColors.text,
                              ),
                            ),
                          ],
                        ),
                        IconButton(
                          icon: const Icon(Icons.close, color: AppColors.textMuted, size: 20),
                          visualDensity: VisualDensity.compact,
                          padding: EdgeInsets.zero,
                          constraints: const BoxConstraints(),
                          onPressed: () => Navigator.of(context).pop(),
                        ),
                      ],
                    ),
                  ],
                ),
              ),

              // 2. Status Card (Always visible at the top, no tabs)
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 12, 16, 8),
                child: Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: isConnected
                        ? AppColors.success.withOpacity(0.12)
                        : (isConnecting
                            ? AppColors.primary.withOpacity(0.12)
                            : AppColors.surface),
                    borderRadius: BorderRadius.circular(14),
                    border: Border.all(
                      color: isConnected
                          ? AppColors.success.withOpacity(0.5)
                          : (isConnecting
                              ? AppColors.primary.withOpacity(0.5)
                              : AppColors.border),
                    ),
                  ),
                  child: Column(
                    children: [
                      Row(
                        children: [
                          Container(
                            padding: const EdgeInsets.all(8),
                            decoration: BoxDecoration(
                              color: (isConnected
                                      ? AppColors.success
                                      : (isConnecting
                                          ? AppColors.primary
                                          : AppColors.textMuted))
                                  .withOpacity(0.2),
                              shape: BoxShape.circle,
                            ),
                            child: Icon(
                              isConnected
                                  ? Icons.bluetooth_connected
                                  : (isConnecting
                                      ? Icons.bluetooth_searching
                                      : Icons.bluetooth_disabled),
                              color: isConnected
                                  ? AppColors.success
                                  : (isConnecting
                                      ? AppColors.primary
                                      : AppColors.textMuted),
                              size: 20,
                            ),
                          ),
                          const SizedBox(width: 10),
                          Expanded(
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Text(
                                  isConnected
                                      ? 'Đã kết nối: $connectedName'
                                      : (isConnecting
                                          ? 'Đang kết nối tới xe...'
                                          : 'Chưa kết nối Bluetooth xe'),
                                  style: const TextStyle(
                                    fontSize: 13,
                                    fontWeight: FontWeight.bold,
                                    color: AppColors.text,
                                  ),
                                ),
                                const SizedBox(height: 2),
                                Text(
                                  isConnected
                                      ? 'MAC: $connectedMac • Kênh SPP 1'
                                      : (connectedMac.isNotEmpty && connectedMac != 'Chưa có MAC'
                                          ? 'MAC đã lưu: $connectedMac'
                                          : 'Sẵn sàng tự động kết nối qua RFCOMM'),
                                  style: const TextStyle(
                                    fontSize: 11,
                                    color: AppColors.textSecondary,
                                  ),
                                ),
                              ],
                            ),
                          ),
                          const SizedBox(width: 8),
                          if (isConnected)
                            OutlinedButton(
                              onPressed: () => state.disconnectPi(),
                              style: OutlinedButton.styleFrom(
                                foregroundColor: AppColors.danger,
                                side: const BorderSide(color: AppColors.danger),
                                padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                                minimumSize: const Size(0, 32),
                                shape: RoundedRectangleBorder(
                                  borderRadius: BorderRadius.circular(8),
                                ),
                              ),
                              child: const Text('Ngắt', style: TextStyle(fontSize: 11, fontWeight: FontWeight.bold)),
                            )
                          else
                            ElevatedButton.icon(
                              onPressed: isConnecting ? null : () => state.matchWithPi(),
                              icon: isConnecting
                                  ? const SizedBox(
                                      width: 12,
                                      height: 12,
                                      child: CircularProgressIndicator(
                                        strokeWidth: 2,
                                        valueColor: AlwaysStoppedAnimation(Colors.black),
                                      ),
                                    )
                                  : const Icon(Icons.sync, size: 14),
                              label: Text(
                                isConnecting ? 'Đang thử' : 'Kết nối',
                                style: const TextStyle(fontSize: 11, fontWeight: FontWeight.bold),
                              ),
                              style: ElevatedButton.styleFrom(
                                backgroundColor: AppColors.primary,
                                foregroundColor: Colors.black,
                                padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                                minimumSize: const Size(0, 32),
                                shape: RoundedRectangleBorder(
                                  borderRadius: BorderRadius.circular(8),
                                ),
                              ),
                            ),
                        ],
                      ),
                      const SizedBox(height: 8),
                      // Telemetry Subsystem Badges
                      Row(
                        children: [
                          _buildMiniBadge(
                            icon: Icons.settings_ethernet,
                            label: isConnected ? 'RFCOMM Online' : 'RFCOMM Chờ',
                            color: isConnected ? AppColors.success : AppColors.textMuted,
                          ),
                          const SizedBox(width: 6),
                          _buildMiniBadge(
                            icon: Icons.tire_repair,
                            label: state.isTpmsEnabled ? 'TPMS Bật' : 'TPMS Tắt',
                            color: state.isTpmsEnabled ? AppColors.primary : AppColors.textMuted,
                          ),
                          const SizedBox(width: 6),
                          _buildMiniBadge(
                            icon: Icons.history,
                            label: '${state.btLogs.length} logs',
                            color: AppColors.secondary,
                          ),
                        ],
                      ),
                    ],
                  ),
                ),
              ),

              // 3. Live Logs Viewer (Expanded, taking remaining vertical area)
              Expanded(
                child: BtLogsTab(
                  state: state,
                  onRequestLogs: _requestBluetoothLogs,
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildMiniBadge({
    required IconData icon,
    required String label,
    required Color color,
  }) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 3),
      decoration: BoxDecoration(
        color: color.withOpacity(0.12),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: color.withOpacity(0.3)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 11, color: color),
          const SizedBox(width: 4),
          Text(
            label,
            style: TextStyle(
              fontSize: 10,
              fontWeight: FontWeight.w600,
              color: color,
            ),
          ),
        ],
      ),
    );
  }
}
