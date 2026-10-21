import 'package:flutter/material.dart';
import '../../theme/app_theme.dart';

/// Banner displaying real-time Bluetooth connection state for TPMS telemetry.
class TpmsStatusBanner extends StatelessWidget {
  final bool isConnected;
  final VoidCallback onSync;

  const TpmsStatusBanner({
    super.key,
    required this.isConnected,
    required this.onSync,
  });

  @override
  Widget build(BuildContext context) {
    final statusColor = isConnected ? AppColors.success : AppColors.warning;

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
      margin: const EdgeInsets.only(bottom: 12),
      decoration: BoxDecoration(
        color: statusColor.withOpacity(0.12),
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: statusColor, width: 1),
      ),
      child: Row(
        children: [
          Icon(
            isConnected ? Icons.bluetooth_connected : Icons.bluetooth_disabled,
            size: 16,
            color: statusColor,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              isConnected
                  ? 'Dữ liệu TPMS được cập nhật qua Bluetooth từ Raspberry Pi'
                  : 'Chưa kết nối Bluetooth (Đang hiển thị trạng thái ngắt kết nối)',
              style: TextStyle(
                fontSize: 11,
                fontWeight: FontWeight.w500,
                color: statusColor,
              ),
            ),
          ),
          if (isConnected)
            IconButton(
              icon: const Icon(Icons.sync, size: 18, color: AppColors.primary),
              tooltip: 'Đồng bộ từ xe',
              padding: EdgeInsets.zero,
              constraints: const BoxConstraints(),
              onPressed: onSync,
            ),
        ],
      ),
    );
  }
}
