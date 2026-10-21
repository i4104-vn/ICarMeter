import 'package:flutter/material.dart';
import '../../theme/app_theme.dart';
import '../../models/bluetooth_device.dart';
import '../../services/app_state.dart';
import 'bt_device_tile.dart';

/// Tab displaying connected status, paired devices, and wireless discovery scanner.
class BtDevicesTab extends StatelessWidget {
  final AppState state;

  const BtDevicesTab({super.key, required this.state});

  @override
  Widget build(BuildContext context) {
    final isConnected = state.isBtConnected;

    return ListView(
      padding: const EdgeInsets.fromLTRB(16, 14, 16, 32),
      children: [
        // Connection Status Banner
        Container(
          padding: const EdgeInsets.all(14),
          decoration: BoxDecoration(
            color: isConnected
                ? AppColors.success.withOpacity(0.12)
                : AppColors.surface,
            borderRadius: BorderRadius.circular(16),
            border: Border.all(
              color: isConnected
                  ? AppColors.success.withOpacity(0.5)
                  : AppColors.border,
            ),
          ),
          child: Row(
            children: [
              Container(
                padding: const EdgeInsets.all(10),
                decoration: BoxDecoration(
                  color: (isConnected ? AppColors.success : AppColors.textMuted)
                      .withOpacity(0.2),
                  shape: BoxShape.circle,
                ),
                child: Icon(
                  isConnected ? Icons.bluetooth_connected : Icons.bluetooth_disabled,
                  color: isConnected ? AppColors.success : AppColors.textMuted,
                  size: 24,
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      isConnected
                          ? 'Đã kết nối: ${state.connectedName.isNotEmpty ? state.connectedName : state.connectedMac}'
                          : 'Chưa kết nối Bluetooth',
                      style: const TextStyle(
                        fontSize: 14,
                        fontWeight: FontWeight.bold,
                        color: AppColors.text,
                      ),
                    ),
                    const SizedBox(height: 2),
                    Text(
                      isConnected
                          ? 'MAC: ${state.connectedMac}'
                          : 'Chọn thiết bị từ danh sách bên dưới để kết nối.',
                      style: const TextStyle(fontSize: 11, color: AppColors.textSecondary),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),

        const SizedBox(height: 20),

        // Vehicle Hub (Pi) Section if identified
        if (state.btCoordinator.piMac.isNotEmpty) ...[
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              const Row(
                children: [
                  Icon(Icons.directions_car, size: 18, color: AppColors.secondary),
                  SizedBox(width: 6),
                  Text(
                    'Thiết bị xe (ICarMeter)',
                    style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold, color: AppColors.text),
                  ),
                ],
              ),
              if (state.isWifiConnected)
                IconButton(
                  onPressed: () => state.matchWithPi(),
                  icon: const Icon(Icons.sync, size: 20, color: AppColors.secondary),
                  tooltip: 'Khớp nối lại với xe',
                ),
            ],
          ),
          const SizedBox(height: 6),
          BtDeviceTile(
            device: state.btCoordinator.piDevice ??
                BtDeviceInfo(
                  mac: state.btCoordinator.piMac,
                  name: state.btCoordinator.piName,
                  isBonded: state.pairedDevices
                      .any((d) => d.mac.toUpperCase() == state.btCoordinator.piMac.toUpperCase()),
                ),
            isPaired: state.pairedDevices
                .any((d) => d.mac.toUpperCase() == state.btCoordinator.piMac.toUpperCase()),
            appState: state,
          ),
          const SizedBox(height: 20),
        ],

        // Paired Devices Header
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            const Row(
              children: [
                Icon(Icons.link, size: 18, color: AppColors.primary),
                SizedBox(width: 6),
                Text(
                  'Thiết bị đã ghép nối',
                  style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold, color: AppColors.text),
                ),
              ],
            ),
            IconButton(
              onPressed: () => state.loadPairedDevices(),
              icon: const Icon(Icons.refresh, size: 20, color: AppColors.primary),
              tooltip: 'Làm mới danh sách',
            ),
          ],
        ),
        const SizedBox(height: 6),
        if (state.pairedDevices.isEmpty)
          Container(
            padding: const EdgeInsets.all(16),
            alignment: Alignment.center,
            child: const Text(
              'Chưa có thiết bị nào được ghép nối.',
              style: TextStyle(fontSize: 12, color: AppColors.textMuted),
            ),
          )
        else
          ...state.pairedDevices.map((d) => BtDeviceTile(
                device: d,
                isPaired: true,
                appState: state,
              )),

        const SizedBox(height: 20),

        // Device Discovery Header
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            const Row(
              children: [
                Icon(Icons.search, size: 18, color: AppColors.secondary),
                SizedBox(width: 6),
                Text(
                  'Dò tìm thiết bị lân cận',
                  style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold, color: AppColors.text),
                ),
              ],
            ),
            ElevatedButton.icon(
              onPressed: state.isDiscovering
                  ? () => state.stopDiscovery()
                  : () => state.startDiscovery(),
              icon: state.isDiscovering
                  ? const SizedBox(
                      width: 14,
                      height: 14,
                      child: CircularProgressIndicator(
                        strokeWidth: 2,
                        valueColor: AlwaysStoppedAnimation(Colors.black),
                      ),
                    )
                  : const Icon(Icons.radar, size: 16),
              label: Text(
                state.isDiscovering ? 'Dừng quét' : 'Tìm thiết bị',
                style: const TextStyle(fontSize: 12, fontWeight: FontWeight.bold),
              ),
              style: ElevatedButton.styleFrom(
                backgroundColor: state.isDiscovering ? AppColors.warning : AppColors.primary,
                foregroundColor: Colors.black,
                padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
                minimumSize: const Size(0, 36),
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
              ),
            ),
          ],
        ),
        const SizedBox(height: 8),

        if (state.discoveredDevices.isEmpty)
          Container(
            padding: const EdgeInsets.all(20),
            alignment: Alignment.center,
            child: Text(
              state.isDiscovering
                  ? 'Đang dò quét các thiết bị Bluetooth xung quanh...'
                  : 'Bấm "Tìm thiết bị" để quét các thiết bị lân cận.',
              style: const TextStyle(fontSize: 12, color: AppColors.textMuted),
            ),
          )
        else
          ...state.discoveredDevices.map((d) => BtDeviceTile(
                device: d,
                isPaired: false,
                appState: state,
              )),
      ],
    );
  }
}
