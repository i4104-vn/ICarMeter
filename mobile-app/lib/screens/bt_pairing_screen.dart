import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../services/app_state.dart';
import '../models/bluetooth_device.dart';

class BtPairingScreen extends StatefulWidget {
  final AppState appState;

  const BtPairingScreen({super.key, required this.appState});

  @override
  State<BtPairingScreen> createState() => _BtPairingScreenState();
}

class _BtPairingScreenState extends State<BtPairingScreen> {
  late TextEditingController _macCtrl;

  @override
  void initState() {
    super.initState();
    _macCtrl = TextEditingController(text: widget.appState.storage.piMac);
  }

  @override
  void dispose() {
    _macCtrl.dispose();
    super.dispose();
  }

  Widget _buildDeviceItem(BtDeviceInfo device, {required bool isPaired}) {
    final isConnected = widget.appState.isBtConnected &&
        widget.appState.connectedMac.toUpperCase() == device.mac.toUpperCase();

    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
      decoration: BoxDecoration(
        color: AppColors.surface,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(
          color: isConnected ? AppColors.success.withOpacity(0.6) : AppColors.border,
          width: isConnected ? 1.5 : 1.0,
        ),
      ),
      child: Row(
        children: [
          Container(
            padding: const EdgeInsets.all(8),
            decoration: BoxDecoration(
              color: (isConnected ? AppColors.success : AppColors.primary).withOpacity(0.12),
              borderRadius: BorderRadius.circular(10),
            ),
            child: Icon(
              Icons.bluetooth,
              color: isConnected ? AppColors.success : AppColors.primary,
              size: 20,
            ),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  device.name.isNotEmpty ? device.name : 'Thiết bị không tên',
                  style: const TextStyle(
                    fontSize: 14,
                    fontWeight: FontWeight.bold,
                    color: AppColors.text,
                  ),
                ),
                const SizedBox(height: 2),
                Text(
                  device.mac,
                  style: const TextStyle(fontSize: 11, color: AppColors.textSecondary, fontFamily: 'monospace'),
                ),
              ],
            ),
          ),
          if (isConnected)
            OutlinedButton(
              onPressed: () => widget.appState.disconnectPi(),
              style: OutlinedButton.styleFrom(
                foregroundColor: AppColors.danger,
                side: const BorderSide(color: AppColors.danger),
                padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                minimumSize: const Size(0, 34),
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
              ),
              child: const Text('Ngắt kết nối', style: TextStyle(fontSize: 12)),
            )
          else if (isPaired)
            ElevatedButton(
              onPressed: () => widget.appState.connectToPi(device.mac),
              style: ElevatedButton.styleFrom(
                backgroundColor: AppColors.primary,
                foregroundColor: Colors.black,
                padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
                minimumSize: const Size(0, 34),
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
              ),
              child: const Text('Kết nối SPP', style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold)),
            )
          else
            ElevatedButton(
              onPressed: () => widget.appState.pairDevice(device.mac),
              style: ElevatedButton.styleFrom(
                backgroundColor: AppColors.secondary,
                foregroundColor: Colors.white,
                padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
                minimumSize: const Size(0, 34),
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
              ),
              child: const Text('Ghép nối', style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold)),
            ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final state = widget.appState;
    final isConnected = state.isBtConnected;

    return SingleChildScrollView(
      physics: const BouncingScrollPhysics(),
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Connection Status Card
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              gradient: isConnected ? AppColors.cardGradient : null,
              color: isConnected ? null : AppColors.surface,
              borderRadius: BorderRadius.circular(16),
              border: Border.all(
                color: isConnected ? AppColors.success.withOpacity(0.5) : AppColors.border,
                width: 1.2,
              ),
            ),
            child: Row(
              children: [
                Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: (isConnected ? AppColors.success : AppColors.textMuted).withOpacity(0.15),
                    shape: BoxShape.circle,
                  ),
                  child: Icon(
                    isConnected ? Icons.bluetooth_connected : Icons.bluetooth_disabled,
                    color: isConnected ? AppColors.success : AppColors.textMuted,
                    size: 26,
                  ),
                ),
                const SizedBox(width: 14),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        isConnected ? 'Đang kết nối SPP' : 'Chưa kết nối SPP',
                        style: const TextStyle(fontSize: 15, fontWeight: FontWeight.bold, color: AppColors.text),
                      ),
                      const SizedBox(height: 2),
                      Text(
                        isConnected
                            ? '${state.connectedName} (${state.connectedMac})'
                            : 'Kênh Bluetooth luôn duy trì sẵn sàng kết nối.',
                        style: const TextStyle(fontSize: 11, color: AppColors.textSecondary),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),

          const SizedBox(height: 16),

          // Manual MAC Connect Input
          Container(
            padding: const EdgeInsets.all(14),
            decoration: BoxDecoration(
              color: AppColors.surface,
              borderRadius: BorderRadius.circular(14),
              border: Border.all(color: AppColors.border, width: 1),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  'Địa chỉ MAC của Raspberry Pi',
                  style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600, color: AppColors.textSecondary),
                ),
                const SizedBox(height: 8),
                Row(
                  children: [
                    Expanded(
                      child: TextField(
                        controller: _macCtrl,
                        style: const TextStyle(fontSize: 13, color: AppColors.text, fontFamily: 'monospace'),
                        decoration: InputDecoration(
                          hintText: 'B8:27:EB:XX:XX:XX',
                          hintStyle: const TextStyle(color: AppColors.textMuted),
                          isDense: true,
                          filled: true,
                          fillColor: AppColors.background,
                          contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
                          border: OutlineInputBorder(
                            borderRadius: BorderRadius.circular(10),
                            borderSide: const BorderSide(color: AppColors.border),
                          ),
                          enabledBorder: OutlineInputBorder(
                            borderRadius: BorderRadius.circular(10),
                            borderSide: const BorderSide(color: AppColors.border),
                          ),
                          focusedBorder: OutlineInputBorder(
                            borderRadius: BorderRadius.circular(10),
                            borderSide: const BorderSide(color: AppColors.primary),
                          ),
                        ),
                      ),
                    ),
                    const SizedBox(width: 10),
                    ElevatedButton(
                      onPressed: () {
                        final mac = _macCtrl.text.trim();
                        if (mac.isNotEmpty) {
                          widget.appState.connectToPi(mac);
                        }
                      },
                      style: ElevatedButton.styleFrom(
                        backgroundColor: AppColors.primary,
                        foregroundColor: Colors.black,
                        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
                        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                      ),
                      child: const Text('Kết nối', style: TextStyle(fontWeight: FontWeight.bold)),
                    ),
                  ],
                ),
              ],
            ),
          ),

          const SizedBox(height: 20),

          // Paired Devices List
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              const Text(
                'Thiết bị đã ghép nối trên điện thoại',
                style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold, color: AppColors.text),
              ),
              IconButton(
                onPressed: () => widget.appState.loadPairedDevices(),
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
                'Không tìm thấy thiết bị nào đã ghép nối.',
                style: TextStyle(fontSize: 12, color: AppColors.textMuted),
              ),
            )
          else
            ...state.pairedDevices.map((d) => _buildDeviceItem(d, isPaired: true)),

          const SizedBox(height: 20),

          // Nearby Discovered Devices
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              const Text(
                'Quét tìm thiết bị lân cận',
                style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold, color: AppColors.text),
              ),
              ElevatedButton.icon(
                onPressed: state.isDiscovering
                    ? () => widget.appState.stopDiscovery()
                    : () => widget.appState.startDiscovery(),
                icon: state.isDiscovering
                    ? const SizedBox(
                        width: 14,
                        height: 14,
                        child: CircularProgressIndicator(strokeWidth: 2, valueColor: AlwaysStoppedAnimation(Colors.black)),
                      )
                    : const Icon(Icons.search, size: 16),
                label: Text(
                  state.isDiscovering ? 'Dừng quét' : 'Quét ngay',
                  style: const TextStyle(fontSize: 12, fontWeight: FontWeight.bold),
                ),
                style: ElevatedButton.styleFrom(
                  backgroundColor: state.isDiscovering ? AppColors.warning : AppColors.primary,
                  foregroundColor: Colors.black,
                  padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                  minimumSize: const Size(0, 34),
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
                    : 'Nhấn "Quét ngay" để tìm Raspberry Pi.',
                style: const TextStyle(fontSize: 12, color: AppColors.textMuted),
              ),
            )
          else
            ...state.discoveredDevices.map((d) => _buildDeviceItem(d, isPaired: false)),
        ],
      ),
    );
  }
}
