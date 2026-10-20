import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../services/app_state.dart';
import '../models/bluetooth_device.dart';

void showBluetoothModal(BuildContext context, AppState appState, {VoidCallback? onOpenWifi}) {
  showModalBottomSheet(
    context: context,
    isScrollControlled: true,
    backgroundColor: Colors.transparent,
    builder: (ctx) => BtPairingModal(appState: appState, onOpenWifi: onOpenWifi),
  );
}

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
  final TextEditingController _logSearchCtrl = TextEditingController();
  final ScrollController _logScrollCtrl = ScrollController();
  bool _autoScroll = true;
  String _logFilter = '';

  @override
  void initState() {
    super.initState();
    _logSearchCtrl.addListener(() {
      setState(() {
        _logFilter = _logSearchCtrl.text.trim().toLowerCase();
      });
    });
    // Request fresh status and logs
    if (widget.appState.isWifiConnected) {
      widget.appState.loadPairedDevices();
      _requestLogsFromPi();
    }
  }

  @override
  void dispose() {
    _logSearchCtrl.dispose();
    _logScrollCtrl.dispose();
    super.dispose();
  }

  void _requestLogsFromPi() {
    final cmd = {'type': 'get_bt_logs'};
    if (widget.appState.isBtConnected) {
      widget.appState.sendBtCommand(cmd);
    }
    widget.appState.wifiWs.send(cmd);
  }

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

  Color _getLogColor(String line) {
    if (line.contains('[BT-SPP]')) return AppColors.primary;
    if (line.contains('[BT-PAIR]') || line.contains('[PI-PAIR]')) return AppColors.warning;
    if (line.contains('[HUB-LOG]') || line.contains('[PI-BT]')) return AppColors.success;
    if (line.contains('error') || line.contains('Error') || line.contains('FAILED')) {
      return AppColors.danger;
    }
    return AppColors.textSecondary;
  }

  Widget _buildDeviceItem(BtDeviceInfo device, {required bool isPaired}) {
    final state = widget.appState;
    final isConnected = state.isBtConnected &&
        state.connectedMac.toUpperCase() == device.mac.toUpperCase();
    final isConnectingThis = state.isBtConnecting &&
        state.storage.piMac.toUpperCase() == device.mac.toUpperCase();
    final displayName = _getDeviceDisplayName(device);

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
              state.connectToPi(device.mac);
            } else {
              state.pairDevice(device.mac);
            }
          },
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
            child: Row(
              children: [
                Container(
                  padding: const EdgeInsets.all(9),
                  decoration: BoxDecoration(
                    color: (isConnected
                            ? AppColors.success
                            : (isConnectingThis ? AppColors.warning : AppColors.primary))
                        .withOpacity(0.15),
                    borderRadius: BorderRadius.circular(10),
                  ),
                  child: Icon(
                    isConnected ? Icons.bluetooth_connected : Icons.bluetooth,
                    color: isConnected
                        ? AppColors.success
                        : (isConnectingThis ? AppColors.warning : AppColors.primary),
                    size: 22,
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      // Tên thiết bị hiển thị to, rõ ràng
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
                                style: const TextStyle(fontSize: 10, color: AppColors.textMuted),
                              ),
                            ),
                          ],
                        ],
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
                      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                      minimumSize: const Size(0, 36),
                      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                    ),
                    child: const Text('Ngắt nối', style: TextStyle(fontSize: 12)),
                  )
                else if (isConnectingThis)
                  const SizedBox(
                    width: 24,
                    height: 24,
                    child: CircularProgressIndicator(strokeWidth: 2, valueColor: AlwaysStoppedAnimation(AppColors.primary)),
                  )
                else if (isPaired)
                  ElevatedButton(
                    onPressed: () => state.connectToPi(device.mac),
                    style: ElevatedButton.styleFrom(
                      backgroundColor: AppColors.primary,
                      foregroundColor: Colors.black,
                      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
                      minimumSize: const Size(0, 36),
                      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                    ),
                    child: const Text('Kết nối', style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold)),
                  )
                else
                  ElevatedButton(
                    onPressed: () => state.pairDevice(device.mac),
                    style: ElevatedButton.styleFrom(
                      backgroundColor: AppColors.secondary,
                      foregroundColor: Colors.white,
                      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
                      minimumSize: const Size(0, 36),
                      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                    ),
                    child: const Text('Ghép nối', style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold)),
                  ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildConnectionTab(AppState state) {
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

        // Danh sách thiết bị đã ghép nối
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
          ...state.pairedDevices.map((d) => _buildDeviceItem(d, isPaired: true)),

        const SizedBox(height: 20),

        // Quét tìm thiết bị xung quanh
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
          ...state.discoveredDevices.map((d) => _buildDeviceItem(d, isPaired: false)),
      ],
    );
  }

  Widget _buildLogsTab(AppState state) {
    final allLogs = state.btLogs;
    final filteredLogs = _logFilter.isEmpty
        ? allLogs
        : allLogs.where((l) => l.toLowerCase().contains(_logFilter)).toList();

    if (_autoScroll && _logScrollCtrl.hasClients) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (_logScrollCtrl.hasClients) {
          _logScrollCtrl.jumpTo(_logScrollCtrl.position.maxScrollExtent);
        }
      });
    }

    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 16),
      child: Column(
        children: [
          // Control Bar
          Row(
            children: [
              Expanded(
                child: TextField(
                  controller: _logSearchCtrl,
                  style: const TextStyle(fontSize: 12, color: AppColors.text),
                  decoration: InputDecoration(
                    hintText: 'Lọc nhật ký (bluetoothctl, pair, spp)...',
                    hintStyle: const TextStyle(fontSize: 12, color: AppColors.textMuted),
                    prefixIcon: const Icon(Icons.search, size: 16, color: AppColors.textMuted),
                    isDense: true,
                    filled: true,
                    fillColor: AppColors.surface,
                    contentPadding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(10),
                      borderSide: const BorderSide(color: AppColors.border),
                    ),
                    enabledBorder: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(10),
                      borderSide: const BorderSide(color: AppColors.border),
                    ),
                  ),
                ),
              ),
              const SizedBox(width: 8),
              IconButton(
                onPressed: _requestLogsFromPi,
                icon: const Icon(Icons.download, color: AppColors.primary, size: 20),
                tooltip: 'Lấy nhật ký mới nhất',
              ),
              IconButton(
                onPressed: () => state.clearLogs(),
                icon: const Icon(Icons.delete_outline, color: AppColors.textMuted, size: 20),
                tooltip: 'Xóa log',
              ),
              IconButton(
                onPressed: () => setState(() => _autoScroll = !_autoScroll),
                icon: Icon(
                  _autoScroll ? Icons.vertical_align_bottom : Icons.pause_circle_outline,
                  color: _autoScroll ? AppColors.primary : AppColors.textMuted,
                  size: 20,
                ),
                tooltip: _autoScroll ? 'Tự cuộn' : 'Dừng cuộn',
              ),
            ],
          ),
          const SizedBox(height: 10),

          // Log Terminal Window
          Expanded(
            child: Container(
              width: double.infinity,
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: const Color(0xFF070A10),
                borderRadius: BorderRadius.circular(14),
                border: Border.all(color: AppColors.border, width: 1.2),
              ),
              child: filteredLogs.isEmpty
                ? const Center(
                    child: Text(
                      'Chưa có dữ liệu nhật ký Bluetooth.\nNhấn nút tải xuống để đồng bộ nhật ký.',
                      style: TextStyle(fontSize: 12, color: AppColors.textMuted),
                      textAlign: TextAlign.center,
                    ),
                  )
                : ListView.builder(
                    controller: _logScrollCtrl,
                    physics: const BouncingScrollPhysics(),
                    itemCount: filteredLogs.length,
                    itemBuilder: (context, index) {
                      final line = filteredLogs[index];
                      final color = _getLogColor(line);

                      return Padding(
                        padding: const EdgeInsets.symmetric(vertical: 2),
                        child: SelectableText(
                          line,
                          style: TextStyle(
                            fontSize: 11,
                            fontFamily: 'monospace',
                            color: color,
                            height: 1.35,
                          ),
                        ),
                      );
                    },
                  ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildWifiRequiredView(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 28, vertical: 32),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Container(
              width: 80,
              height: 80,
              decoration: BoxDecoration(
                color: AppColors.warning.withOpacity(0.12),
                shape: BoxShape.circle,
                border: Border.all(color: AppColors.warning.withOpacity(0.35), width: 1.5),
                boxShadow: [
                  BoxShadow(
                    color: AppColors.warning.withOpacity(0.2),
                    blurRadius: 24,
                    offset: const Offset(0, 4),
                  ),
                ],
              ),
              child: const Icon(
                Icons.wifi_off_rounded,
                color: AppColors.warning,
                size: 38,
              ),
            ),
            const SizedBox(height: 20),
            const Text(
              'Yêu cầu kết nối Wi-Fi',
              style: TextStyle(
                fontSize: 18,
                fontWeight: FontWeight.bold,
                color: AppColors.text,
                letterSpacing: 0.3,
              ),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 10),
            const Text(
              'Để quản lý, tìm kiếm thiết bị và ghép nối Bluetooth cho xe, bạn bắt buộc phải kết nối vào mạng Wi-Fi của xe trước.',
              style: TextStyle(
                fontSize: 13,
                color: AppColors.textSecondary,
                height: 1.5,
              ),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 28),
            ElevatedButton.icon(
              onPressed: () {
                Navigator.of(context).pop();
                widget.onOpenWifi?.call();
              },
              icon: const Icon(Icons.wifi, size: 18),
              label: const Text(
                'Kết nối Wi-Fi ngay',
                style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold),
              ),
              style: ElevatedButton.styleFrom(
                backgroundColor: AppColors.primary,
                foregroundColor: Colors.black,
                padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 14),
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(14)),
                elevation: 3,
                shadowColor: AppColors.primary.withOpacity(0.4),
              ),
            ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.appState,
      builder: (context, _) {
        final state = widget.appState;

        return DraggableScrollableSheet(
          initialChildSize: 0.88,
          minChildSize: 0.5,
          maxChildSize: 0.95,
          builder: (context, scrollController) {
            return Container(
              decoration: const BoxDecoration(
                color: Color(0xFF0F172A),
                borderRadius: BorderRadius.vertical(top: Radius.circular(24)),
                border: Border(
                  top: BorderSide(color: AppColors.border, width: 1.5),
                ),
              ),
              child: Column(
                children: [
                  // Drag Handle
                  Center(
                    child: Container(
                      margin: const EdgeInsets.only(top: 10, bottom: 8),
                      width: 44,
                      height: 4,
                      decoration: BoxDecoration(
                        color: AppColors.textMuted.withOpacity(0.4),
                        borderRadius: BorderRadius.circular(2),
                      ),
                    ),
                  ),

                  // Modal Header
                  Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
                    child: Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        if (state.isWifiConnected)
                          // Custom 2-Tab Switcher
                          Container(
                            padding: const EdgeInsets.all(4),
                            decoration: BoxDecoration(
                              color: AppColors.background,
                              borderRadius: BorderRadius.circular(12),
                              border: Border.all(color: AppColors.border),
                            ),
                            child: Row(
                              children: [
                                GestureDetector(
                                  onTap: () => setState(() => _selectedTab = 0),
                                  child: AnimatedContainer(
                                    duration: const Duration(milliseconds: 200),
                                    curve: Curves.easeOutCubic,
                                    padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
                                    decoration: BoxDecoration(
                                      color: _selectedTab == 0 ? AppColors.primary : Colors.transparent,
                                      borderRadius: BorderRadius.circular(8),
                                    ),
                                    child: Row(
                                      children: [
                                        Icon(
                                          Icons.bluetooth,
                                          size: 16,
                                          color: _selectedTab == 0 ? Colors.black : AppColors.textSecondary,
                                        ),
                                        const SizedBox(width: 6),
                                        Text(
                                          'Kết nối',
                                          style: TextStyle(
                                            fontSize: 13,
                                            fontWeight: _selectedTab == 0 ? FontWeight.bold : FontWeight.normal,
                                            color: _selectedTab == 0 ? Colors.black : AppColors.textSecondary,
                                          ),
                                        ),
                                      ],
                                    ),
                                  ),
                                ),
                                GestureDetector(
                                  onTap: () => setState(() => _selectedTab = 1),
                                  child: AnimatedContainer(
                                    duration: const Duration(milliseconds: 200),
                                    curve: Curves.easeOutCubic,
                                    padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
                                    decoration: BoxDecoration(
                                      color: _selectedTab == 1 ? AppColors.primary : Colors.transparent,
                                      borderRadius: BorderRadius.circular(8),
                                    ),
                                    child: Row(
                                      children: [
                                        Icon(
                                          Icons.terminal,
                                          size: 16,
                                          color: _selectedTab == 1 ? Colors.black : AppColors.textSecondary,
                                        ),
                                        const SizedBox(width: 6),
                                        Text(
                                          'Nhật ký',
                                          style: TextStyle(
                                            fontSize: 13,
                                            fontWeight: _selectedTab == 1 ? FontWeight.bold : FontWeight.normal,
                                            color: _selectedTab == 1 ? Colors.black : AppColors.textSecondary,
                                          ),
                                        ),
                                      ],
                                    ),
                                  ),
                                ),
                              ],
                            ),
                          )
                        else
                          Row(
                            children: [
                              Container(
                                padding: const EdgeInsets.all(7),
                                decoration: BoxDecoration(
                                  color: AppColors.primary.withOpacity(0.12),
                                  borderRadius: BorderRadius.circular(10),
                                ),
                                child: const Icon(Icons.bluetooth, color: AppColors.primary, size: 20),
                              ),
                              const SizedBox(width: 10),
                              const Text(
                                'Cài đặt Bluetooth',
                                style: TextStyle(
                                  fontSize: 16,
                                  fontWeight: FontWeight.bold,
                                  color: AppColors.text,
                                ),
                              ),
                            ],
                          ),

                        IconButton(
                          icon: const Icon(Icons.close, color: AppColors.textSecondary),
                          onPressed: () => Navigator.of(context).pop(),
                        ),
                      ],
                    ),
                  ),

                  const Divider(color: AppColors.border, height: 1),

                  // Tab View Content or Wi-Fi Required Screen
                  Expanded(
                    child: AnimatedSwitcher(
                      duration: const Duration(milliseconds: 250),
                      switchInCurve: Curves.easeOutCubic,
                      switchOutCurve: Curves.easeInCubic,
                      transitionBuilder: (child, animation) {
                        return FadeTransition(
                          opacity: animation,
                          child: child,
                        );
                      },
                      child: !state.isWifiConnected
                          ? _buildWifiRequiredView(context)
                          : KeyedSubtree(
                              key: ValueKey<int>(_selectedTab),
                              child: _selectedTab == 0
                                  ? _buildConnectionTab(state)
                                  : _buildLogsTab(state),
                            ),
                    ),
                  ),
                ],
              ),
            );
          },
        );
      },
    );
  }
}
