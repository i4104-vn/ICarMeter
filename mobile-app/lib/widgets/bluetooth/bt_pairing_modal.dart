import 'package:flutter/material.dart';
import '../../theme/app_theme.dart';
import '../../services/app_state.dart';
import 'bt_devices_tab.dart';
import 'bt_logs_tab.dart';

void showBluetoothModal(BuildContext context, AppState appState, {VoidCallback? onOpenWifi}) {
  showModalBottomSheet(
    context: context,
    isScrollControlled: true,
    backgroundColor: Colors.transparent,
    builder: (ctx) => BtPairingModal(appState: appState, onOpenWifi: onOpenWifi),
  );
}

/// Modal bottom sheet for Bluetooth management: connections and live diagnostic logs.
class BtPairingModal extends StatefulWidget {
  final AppState appState;
  final VoidCallback? onOpenWifi;

  const BtPairingModal({
    super.key,
    required this.appState,
    this.onOpenWifi,
  });

  @override
  State<BtPairingModal> createState() => _BtPairingModalState();
}

class _BtPairingModalState extends State<BtPairingModal> {
  int _selectedTab = 0; // 0: Kết nối, 1: Nhật ký

  @override
  void initState() {
    super.initState();
    if (widget.appState.isWifiConnected) {
      widget.appState.loadPairedDevices();
      _requestBluetoothLogs();
    }
  }

  void _requestBluetoothLogs() {
    final cmd = {'type': 'get_bt_logs'};
    if (widget.appState.isBtConnected) {
      widget.appState.sendBtCommand(cmd);
    }
    widget.appState.wifiWs.send(cmd);
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.appState,
      builder: (context, _) {
        final state = widget.appState;

        return Container(
          height: MediaQuery.of(context).size.height * 0.88,
          decoration: const BoxDecoration(
            color: AppColors.background,
            borderRadius: BorderRadius.vertical(top: Radius.circular(24)),
          ),
          child: Column(
            children: [
              // Modal Header with Drag Handle
              Container(
                padding: const EdgeInsets.fromLTRB(16, 12, 16, 10),
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
                    const SizedBox(height: 12),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        const Row(
                          children: [
                            Icon(Icons.bluetooth, color: AppColors.primary, size: 22),
                            SizedBox(width: 8),
                            Text(
                              'Quản lý kết nối Bluetooth xe',
                              style: TextStyle(
                                fontSize: 16,
                                fontWeight: FontWeight.bold,
                                color: AppColors.text,
                              ),
                            ),
                          ],
                        ),
                        if (widget.onOpenWifi != null)
                          TextButton.icon(
                            onPressed: () {
                              Navigator.of(context).pop();
                              widget.onOpenWifi!();
                            },
                            icon: const Icon(Icons.wifi, size: 16),
                            label: const Text('Wi-Fi Xe', style: TextStyle(fontSize: 12)),
                            style: TextButton.styleFrom(
                              foregroundColor: AppColors.secondary,
                              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                            ),
                          ),
                      ],
                    ),
                    const SizedBox(height: 12),

                    // Custom Segmented Pill Tab Bar
                    Container(
                      padding: const EdgeInsets.all(4),
                      decoration: BoxDecoration(
                        color: AppColors.background,
                        borderRadius: BorderRadius.circular(12),
                        border: Border.all(color: AppColors.border),
                      ),
                      child: Row(
                        children: [
                          Expanded(
                            child: GestureDetector(
                              onTap: () => setState(() => _selectedTab = 0),
                              child: Container(
                                padding: const EdgeInsets.symmetric(vertical: 8),
                                decoration: BoxDecoration(
                                  color: _selectedTab == 0 ? AppColors.surfaceLight : Colors.transparent,
                                  borderRadius: BorderRadius.circular(8),
                                ),
                                alignment: Alignment.center,
                                child: Text(
                                  'Thiết bị & Kết nối',
                                  style: TextStyle(
                                    fontSize: 13,
                                    fontWeight: _selectedTab == 0 ? FontWeight.bold : FontWeight.normal,
                                    color: _selectedTab == 0 ? AppColors.primary : AppColors.textSecondary,
                                  ),
                                ),
                              ),
                            ),
                          ),
                          Expanded(
                            child: GestureDetector(
                              onTap: () {
                                setState(() => _selectedTab = 1);
                                _requestBluetoothLogs();
                              },
                              child: Container(
                                padding: const EdgeInsets.symmetric(vertical: 8),
                                decoration: BoxDecoration(
                                  color: _selectedTab == 1 ? AppColors.surfaceLight : Colors.transparent,
                                  borderRadius: BorderRadius.circular(8),
                                ),
                                alignment: Alignment.center,
                                child: Row(
                                  mainAxisAlignment: MainAxisAlignment.center,
                                  children: [
                                    Text(
                                      'Nhật ký daemon',
                                      style: TextStyle(
                                        fontSize: 13,
                                        fontWeight: _selectedTab == 1 ? FontWeight.bold : FontWeight.normal,
                                        color: _selectedTab == 1 ? AppColors.primary : AppColors.textSecondary,
                                      ),
                                    ),
                                    if (state.btLogs.isNotEmpty) ...[
                                      const SizedBox(width: 6),
                                      Container(
                                        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 1),
                                        decoration: BoxDecoration(
                                          color: AppColors.primary.withOpacity(0.2),
                                          borderRadius: BorderRadius.circular(10),
                                        ),
                                        child: Text(
                                          '${state.btLogs.length}',
                                          style: const TextStyle(fontSize: 10, color: AppColors.primary, fontWeight: FontWeight.bold),
                                        ),
                                      ),
                                    ],
                                  ],
                                ),
                              ),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
              ),

              // Tab Body
              Expanded(
                child: _selectedTab == 0
                    ? BtDevicesTab(state: state)
                    : BtLogsTab(
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
}
