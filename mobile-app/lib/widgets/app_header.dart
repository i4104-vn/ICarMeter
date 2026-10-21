import 'package:flutter/material.dart';

import '../services/app_state.dart';
import '../theme/app_theme.dart';

class AppHeader extends StatelessWidget implements PreferredSizeWidget {
  final AppState appState;
  final VoidCallback? onOpenBluetooth;
  final VoidCallback? onOpenSettings;

  const AppHeader({
    super.key,
    required this.appState,
    this.onOpenBluetooth,
    this.onOpenSettings,
  });

  @override
  Size get preferredSize => const Size.fromHeight(64);

  @override
  Widget build(BuildContext context) {
    final connected = appState.isBtConnected;
    final connecting = appState.isBtConnecting;

    return Container(
      padding: EdgeInsets.only(
        top: MediaQuery.of(context).padding.top + 8,
        left: 16,
        right: 16,
        bottom: 8,
      ),
      decoration: const BoxDecoration(
        color: AppColors.background,
        border: Border(bottom: BorderSide(color: AppColors.border)),
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
                        connected
                            ? (appState.connectedName.isNotEmpty
                                ? appState.connectedName
                                : 'Bluetooth connected')
                            : (connecting ? 'Connecting...' : 'Not connected'),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: TextStyle(
                          fontSize: 11,
                          color: connected
                              ? AppColors.success
                              : (connecting ? AppColors.primary : AppColors.textMuted),
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
              InkWell(
                onTap: onOpenBluetooth,
                borderRadius: BorderRadius.circular(20),
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
                  decoration: BoxDecoration(
                    color: connected
                        ? AppColors.success.withOpacity(0.15)
                        : (connecting
                            ? AppColors.primary.withOpacity(0.15)
                            : AppColors.surface),
                    borderRadius: BorderRadius.circular(20),
                    border: Border.all(
                      color: connected
                          ? AppColors.success.withOpacity(0.6)
                          : (connecting
                              ? AppColors.primary.withOpacity(0.6)
                              : AppColors.border),
                    ),
                  ),
                  child: Row(
                    children: [
                      if (connecting)
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
                          color: connected ? AppColors.success : AppColors.textMuted,
                        ),
                      const SizedBox(width: 4),
                      Text(
                        connected ? 'BT SPP' : 'Bluetooth',
                        style: TextStyle(
                          fontSize: 11,
                          fontWeight: FontWeight.w600,
                          color: connected
                              ? AppColors.success
                              : (connecting ? AppColors.primary : AppColors.textMuted),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
              const SizedBox(width: 8),
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
