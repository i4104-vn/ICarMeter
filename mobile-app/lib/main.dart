import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'screens/dashboard_screen.dart';
import 'screens/tpms_screen.dart';
import 'screens/trips_screen.dart';
import 'services/app_state.dart';
import 'services/storage/storage_service.dart';
import 'theme/app_theme.dart';
import 'widgets/app_header.dart';
import 'widgets/bluetooth/bt_pairing_modal.dart';
import 'widgets/bluetooth/pairing_pin_dialog.dart';
import 'widgets/pill_nav_bar.dart';
import 'widgets/settings/settings_modal.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  SystemChrome.setSystemUIOverlayStyle(
    const SystemUiOverlayStyle(
      statusBarColor: Colors.transparent,
      statusBarIconBrightness: Brightness.light,
      systemNavigationBarColor: AppColors.background,
      systemNavigationBarIconBrightness: Brightness.light,
    ),
  );

  final storage = await StorageService.init();
  runApp(ICarMeterApp(appState: AppState(storage)));
}

class ICarMeterApp extends StatelessWidget {
  final AppState appState;

  const ICarMeterApp({super.key, required this.appState});

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: appState,
      builder: (context, _) => MaterialApp(
        title: 'ICarMeter Companion',
        debugShowCheckedModeBanner: false,
        theme: AppTheme.darkTheme,
        home: MainHomeScreen(appState: appState),
      ),
    );
  }
}

class MainHomeScreen extends StatefulWidget {
  final AppState appState;

  const MainHomeScreen({super.key, required this.appState});

  @override
  State<MainHomeScreen> createState() => _MainHomeScreenState();
}

class _MainHomeScreenState extends State<MainHomeScreen> {
  int _currentIndex = 0;
  late final PageController _pageController;

  @override
  void initState() {
    super.initState();
    _pageController = PageController();
  }

  @override
  void dispose() {
    _pageController.dispose();
    super.dispose();
  }

  void _onSelectTab(int index) {
    if (_currentIndex == index) return;
    setState(() => _currentIndex = index);
    _pageController.animateToPage(
      index,
      duration: const Duration(milliseconds: 380),
      curve: Curves.easeInOutCubic,
    );
  }

  void _openBluetoothModal() {
    showBluetoothModal(context, widget.appState);
  }

  void _openSettingsModal() {
    showSettingsModal(context, widget.appState);
  }

  @override
  Widget build(BuildContext context) {
    final state = widget.appState;
    final showTpms = state.isTpmsEnabled;
    final screens = [
      DashboardScreen(appState: state),
      if (showTpms) TpmsScreen(appState: state),
      TripsScreen(appState: state),
    ];

    if (_currentIndex >= screens.length) {
      _currentIndex = screens.length - 1;
    }

    return Scaffold(
      appBar: AppHeader(
        appState: state,
        onOpenBluetooth: _openBluetoothModal,
        onOpenSettings: _openSettingsModal,
      ),
      body: Stack(
        children: [
          PageView(
            controller: _pageController,
            physics: const NeverScrollableScrollPhysics(),
            onPageChanged: (index) => setState(() => _currentIndex = index),
            children: screens,
          ),
          if (state.pendingPinCode != null && state.pendingPinCode!.isNotEmpty)
            Container(
              color: Colors.black54,
              alignment: Alignment.center,
              child: PairingPinDialog(
                pinCode: state.pendingPinCode!,
                deviceName: state.pendingPinDeviceName ?? '',
                deviceMac: state.pendingPinDeviceMac ?? '',
                onConfirm: state.dismissPinDialog,
              ),
            ),
        ],
      ),
      bottomNavigationBar: PillNavBar(
        currentIndex: _currentIndex,
        onTap: _onSelectTab,
        showTpms: showTpms,
      ),
    );
  }
}
