import 'package:flutter/material.dart';
import '../../theme/app_theme.dart';
import '../../models/bluetooth_device.dart';
import '../../services/app_state.dart';

/// Single Bluetooth device row with status icons, RSSI, and SPP connection actions.
class BtDeviceTile extends StatelessWidget {
  final BtDeviceInfo device;
  final bool isPaired;
  final AppState appState;

  const BtDeviceTile({
    super.key,
    required this.device,
    required this.isPaired,
    required this.appState,
  });

  String _getDeviceDisplayName(BtDeviceInfo device) {
    final name = device.name.trim();
    final mac = device.mac.trim();

    if (name.isNotEmpty && name.toUpperCase() != mac.toUpperCase()) {
      return name;
    }
    if (mac.isNotEmpty) {
      final shortMac = mac.length >= 5 ? mac.substring(mac.length - 5) : mac;
      return 'Thiết bị ($shortMac)';
    }
    return 'Thiết bị không tên';
  }

  @override
  Widget build(BuildContext context) {
    final isConnected = appState.isBtConnected &&
        appState.connectedMac.toUpperCase() == device.mac.toUpperCase();
    final isConnectingThis = appState.isBtConnecting &&
        appState.storage.piMac.toUpperCase() == device.mac.toUpperCase();
    final displayName = _getDeviceDisplayName(device);

    final statusColor = switch ((isConnected, isConnectingThis)) {
      (true, _) => AppColors.success,
      (false, true) => AppColors.warning,
      _ => AppColors.primary,
    };

    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      decoration: BoxDecoration(
        color: AppColors.surface,
        borderRadius: BorderRadius.circular(14),
        border: Border.all(
          color: isConnected
              ? AppColors.success.withOpacity(0.7)
              : (isConnectingThis ? AppColors.primary.withOpacity(0.5) : AppColors.border),
          width: isConnected ? 1.5 : 1.0,
        ),
      ),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          borderRadius: BorderRadius.circular(14),
          onTap: () {
            if (isConnected || isConnectingThis) return;
            if (isPaired) {
              appState.connectToPi(device.mac);
            } else {
              appState.pairDevice(device.mac);
            }
          },
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
            child: Row(
              children: [
                Container(
                  padding: const EdgeInsets.all(9),
                  decoration: BoxDecoration(
                    color: statusColor.withOpacity(0.15),
                    borderRadius: BorderRadius.circular(10),
                  ),
                  child: Icon(
                    isConnected ? Icons.bluetooth_connected : Icons.bluetooth,
                    color: statusColor,
                    size: 22,
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(
                        displayName,
                        style: const TextStyle(
                          fontSize: 15,
                          fontWeight: FontWeight.bold,
                          color: AppColors.text,
                          letterSpacing: 0.2,
                        ),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                      const SizedBox(height: 3),
                      Row(
                        children: [
                          Text(
                            device.mac,
                            style: const TextStyle(
                              fontSize: 11,
                              color: AppColors.textSecondary,
                              fontFamily: 'monospace',
                            ),
                          ),
                          if (device.rssi != -100 && device.rssi != 0) ...[
                            const SizedBox(width: 8),
                            Container(
                              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 1),
                              decoration: BoxDecoration(
                                color: AppColors.surfaceLight,
                                borderRadius: BorderRadius.circular(4),
                              ),
                              child: Text(
                                '${device.rssi} dBm',
                                style: const TextStyle(fontSize: 10, color: AppColors.textSecondary),
                              ),
                            ),
                          ],
                        ],
                      ),
                    ],
                  ),
                ),
                const SizedBox(width: 8),
                switch ((isConnected, isConnectingThis, isPaired)) {
                  (true, _, _) => OutlinedButton(
                      onPressed: () => appState.disconnectPi(),
                      style: OutlinedButton.styleFrom(
                        foregroundColor: AppColors.danger,
                        side: const BorderSide(color: AppColors.danger),
                        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                        minimumSize: const Size(0, 36),
                        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                      ),
                      child: const Text('Ngắt nối', style: TextStyle(fontSize: 12)),
                    ),
                  (false, true, _) => const SizedBox(
                      width: 24,
                      height: 24,
                      child: CircularProgressIndicator(
                        strokeWidth: 2,
                        valueColor: AlwaysStoppedAnimation(AppColors.primary),
                      ),
                    ),
                  (false, false, true) => ElevatedButton(
                      onPressed: () => appState.connectToPi(device.mac),
                      style: ElevatedButton.styleFrom(
                        backgroundColor: AppColors.primary,
                        foregroundColor: Colors.black,
                        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
                        minimumSize: const Size(0, 36),
                        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                      ),
                      child: const Text('Kết nối', style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold)),
                    ),
                  (false, false, false) => ElevatedButton(
                      onPressed: () => appState.pairDevice(device.mac),
                      style: ElevatedButton.styleFrom(
                        backgroundColor: AppColors.secondary,
                        foregroundColor: Colors.white,
                        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
                        minimumSize: const Size(0, 36),
                        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                      ),
                      child: const Text('Ghép nối', style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold)),
                    ),
                },
              ],
            ),
          ),
        ),
      ),
    );
  }
}
