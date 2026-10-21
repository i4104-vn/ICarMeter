import 'package:flutter/material.dart';
import '../theme/app_theme.dart';

class PillNavBarItem {
  final IconData icon;
  final String label;

  const PillNavBarItem({required this.icon, required this.label});
}

class PillNavBar extends StatelessWidget {
  final int currentIndex;
  final ValueChanged<int> onTap;

  static const List<PillNavBarItem> items = [
    PillNavBarItem(icon: Icons.speed, label: 'Đồng hồ'),
    PillNavBarItem(icon: Icons.tire_repair, label: 'Lốp xe'),
    PillNavBarItem(icon: Icons.route, label: 'Chuyến đi'),
  ];

  const PillNavBar({
    super.key,
    required this.currentIndex,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final selectedIndex = currentIndex.clamp(0, items.length - 1);

    return SafeArea(
      top: false,
      child: Container(
        margin: const EdgeInsets.fromLTRB(16, 0, 16, 12),
        padding: const EdgeInsets.all(6),
        decoration: BoxDecoration(
          color: const Color(0xFF131B2E).withOpacity(0.95),
          borderRadius: BorderRadius.circular(26),
          border: Border.all(color: AppColors.border, width: 1.2),
          boxShadow: [
            BoxShadow(
              color: Colors.black.withOpacity(0.4),
              blurRadius: 16,
              offset: const Offset(0, 6),
            ),
          ],
        ),
        child: SizedBox(
          height: 46,
          child: Stack(
            children: [
              AnimatedAlign(
                alignment: Alignment(
                  -1 + (2 * selectedIndex / (items.length - 1)),
                  0,
                ),
                duration: const Duration(milliseconds: 380),
                curve: Curves.easeInOutCubic,
                child: FractionallySizedBox(
                  widthFactor: 1 / items.length,
                  heightFactor: 1,
                  child: Container(
                    margin: const EdgeInsets.symmetric(horizontal: 3),
                    decoration: BoxDecoration(
                      gradient: AppColors.primaryGradient,
                      borderRadius: BorderRadius.circular(20),
                      boxShadow: [
                        BoxShadow(
                          color: AppColors.primary.withOpacity(0.28),
                          blurRadius: 12,
                          offset: const Offset(0, 3),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
              Row(
                children: List.generate(items.length, (index) {
                  final isSelected = index == selectedIndex;
                  final item = items[index];

                  return Expanded(
                    child: Semantics(
                      button: true,
                      selected: isSelected,
                      label: item.label,
                      child: GestureDetector(
                        onTap: () => onTap(index),
                        behavior: HitTestBehavior.opaque,
                        child: SizedBox(
                          height: 46,
                          child: Row(
                            mainAxisAlignment: MainAxisAlignment.center,
                            mainAxisSize: MainAxisSize.min,
                            children: [
                              AnimatedScale(
                                scale: isSelected ? 1.08 : 1,
                                duration: const Duration(milliseconds: 280),
                                curve: Curves.easeOutCubic,
                                child: TweenAnimationBuilder<Color?>(
                                  tween: ColorTween(
                                    end: isSelected
                                        ? Colors.black
                                        : AppColors.textSecondary,
                                  ),
                                  duration: const Duration(milliseconds: 260),
                                  curve: Curves.easeInOut,
                                  builder: (context, color, _) => Icon(
                                    item.icon,
                                    size: 19,
                                    color: color,
                                  ),
                                ),
                              ),
                              const SizedBox(width: 7),
                              AnimatedDefaultTextStyle(
                                duration: const Duration(milliseconds: 220),
                                curve: Curves.easeOut,
                                style: TextStyle(
                                  fontSize: 12.5,
                                  fontWeight: isSelected
                                      ? FontWeight.bold
                                      : FontWeight.w600,
                                  color: isSelected
                                      ? Colors.black
                                      : AppColors.textSecondary,
                                  letterSpacing: 0.2,
                                ),
                                child: Text(item.label),
                              ),
                            ],
                          ),
                        ),
                      ),
                    ),
                  );
                }),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
