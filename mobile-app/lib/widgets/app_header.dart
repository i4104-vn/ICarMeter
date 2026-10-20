import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../services/app_state.dart';

class AppHeader extends StatelessWidget implements PreferredSizeWidget {
  final AppState appState;
  final VoidCallback? onOpenBluetooth;
  final VoidCallback? onOpenWifi;
  final VoidCallback? onOpenSettings;

  const AppHeader({
    super.key,
    required this.appState,
    this.onOpenBluetooth,
    this.onOpenWifi,
    this.onOpenSettings,
  });

  @override
  Size get preferredSize => const Size.fromHeight(64);

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: EdgeInsets.only(
        top: MediaQuery.of(context).padding.top + 8,
        left: 16,
        right: 16,
        bottom: 8,
      ),
      decoration: const BoxDecoration(
        color: AppColors.background,
        border: Border(
          bottom: BorderSide(color: AppColors.border, width: 1),
        ),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Expanded(
            child: Row(
              children: [
                Container(
                  width: 38,
                  height: 38,
                  decoration: BoxDecoration(
                    gradient: AppColors.primaryGradient,
                    borderRadius: BorderRadius.circular(10),
                    boxShadow: [
                      BoxShadow(
                        color: AppColors.primary.withOpacity(0.35),
                        blurRadius: 10,
                        offset: const Offset(0, 2),
                      ),
                    ],
                  ),
                  child: const Icon(Icons.directions_car, color: Colors.black, size: 22),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      const Text(
                        'ICarMeter',
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: TextStyle(
                          fontSize: 18,
                          fontWeight: FontWeight.bold,
                          letterSpacing: 0.5,
                          color: AppColors.text,
                        ),
                      ),
                      Text(
                        appState.isBtConnected
                            ? (appState.connectedName.isNotEmpty
                                ? appState.connectedName
                                : 'Đã kết nối Bluetooth')
                            : (appState.isBtConnecting
                                ? 'Đang kết nối...'
                                : 'Chưa kết nối'),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: TextStyle(
                          fontSize: 11,
                          color: appState.isBtConnected
                              ? AppColors.success
                              : (appState.isBtConnecting ? AppColors.primary : AppColors.textMuted),
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(width: 8),
          Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              // Wi-Fi Status Badge
              InkWell(
                onTap: onOpenWifi,
                borderRadius: BorderRadius.circular(20),
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
                  decoration: BoxDecoration(
                    color: appState.isWifiConnected
                        ? AppColors.info.withOpacity(0.15)
                        : AppColors.surface,
                    borderRadius: BorderRadius.circular(20),
                    border: Border.all(
                      color: appState.isWifiConnected
                          ? AppColors.info.withOpacity(0.6)
                          : AppColors.border,
                    ),
                  ),
                  child: Row(
                    children: [
                      Icon(
                        Icons.wifi,
                        size: 14,
                        color: appState.isWifiConnected ? AppColors.info : AppColors.textMuted,
                      ),
                      const SizedBox(width: 4),
                      Text(
                        appState.isWifiConnected ? 'Wi-Fi' : 'Offline',
                        style: TextStyle(
                          fontSize: 11,
                          fontWeight: FontWeight.w600,
                          color: appState.isWifiConnected ? AppColors.info : AppColors.textMuted,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              const SizedBox(width: 8),

              // Bluetooth Status Badge
              InkWell(
                onTap: onOpenBluetooth,
                borderRadius: BorderRadius.circular(20),
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
                  decoration: BoxDecoration(
                    color: appState.isBtConnected
                        ? AppColors.success.withOpacity(0.15)
                        : (appState.isBtConnecting
                            ? AppColors.primary.withOpacity(0.15)
                            : AppColors.surface),
                    borderRadius: BorderRadius.circular(20),
                    border: Border.all(
                      color: appState.isBtConnected
                          ? AppColors.success.withOpacity(0.6)
                          : (appState.isBtConnecting
                              ? AppColors.primary.withOpacity(0.6)
                              : AppColors.border),
                    ),
                  ),
                  child: Row(
                    children: [
                      if (appState.isBtConnecting)
                        const SizedBox(
                          width: 12,
                          height: 12,
                          child: CircularProgressIndicator(
                            strokeWidth: 2,
                            valueColor: AlwaysStoppedAnimation(AppColors.primary),
                          ),
                        )
                      else
                        Icon(
                          Icons.bluetooth,
                          size: 14,
                          color: appState.isBtConnected
                              ? AppColors.success
                              : AppColors.textMuted,
                        ),
                      const SizedBox(width: 4),
                      Text(
                        appState.isBtConnected ? 'BT SPP' : 'Chưa nối',
                        style: TextStyle(
                          fontSize: 11,
                          fontWeight: FontWeight.w600,
                          color: appState.isBtConnected
                              ? AppColors.success
                              : (appState.isBtConnecting
                                  ? AppColors.primary
                                  : AppColors.textMuted),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              const SizedBox(width: 8),

              // Settings Icon Button
              InkWell(
                onTap: onOpenSettings,
                borderRadius: BorderRadius.circular(20),
                child: Container(
                  padding: const EdgeInsets.all(7),
                  decoration: BoxDecoration(
                    color: AppColors.surface,
                    borderRadius: BorderRadius.circular(20),
                    border: Border.all(color: AppColors.border),
                  ),
                  child: const Icon(
                    Icons.settings_outlined,
                    size: 15,
                    color: AppColors.textSecondary,
                  ),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
