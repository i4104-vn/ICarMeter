import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'theme/app_theme.dart';
import 'services/storage/storage_service.dart';
import 'services/app_state.dart';
import 'widgets/app_header.dart';
import 'widgets/pill_nav_bar.dart';
import 'widgets/bluetooth/bt_pairing_modal.dart';
import 'widgets/settings/settings_modal.dart';
import 'widgets/bluetooth/pairing_pin_dialog.dart';
import 'screens/dashboard_screen.dart';
import 'screens/tpms_screen.dart';
import 'screens/trips_screen.dart';
import 'screens/wifi_setup_screen.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Set system UI overlay style
  SystemChrome.setSystemUIOverlayStyle(
    const SystemUiOverlayStyle(
      statusBarColor: Colors.transparent,
      statusBarIconBrightness: Brightness.light,
      systemNavigationBarColor: AppColors.background,
      systemNavigationBarIconBrightness: Brightness.light,
    ),
  );

  final storage = await StorageService.init();
  final appState = AppState(storage);

  runApp(ICarMeterApp(appState: appState));
}

class ICarMeterApp extends StatelessWidget {
  final AppState appState;

  const ICarMeterApp({super.key, required this.appState});

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: appState,
      builder: (context, _) {
        return MaterialApp(
          title: 'ICarMeter Companion',
          debugShowCheckedModeBanner: false,
          theme: AppTheme.darkTheme,
          home: MainHomeScreen(appState: appState),
        );
      },
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
    _pageController = PageController(initialPage: _currentIndex);
  }

  @override
  void dispose() {
    _pageController.dispose();
    super.dispose();
  }

  void _onSelectTab(int index) {
    if (_currentIndex == index) return;
    setState(() {
      _currentIndex = index;
    });
    _pageController.animateToPage(
      index,
      duration: const Duration(milliseconds: 380),
      curve: Curves.easeInOutCubic,
    );
  }

  void _openBluetoothModal() {
    showBluetoothModal(
      context,
      widget.appState,
      onOpenWifi: _openWifiSetupModal,
    );
  }

  void _openSettingsModal() {
    showSettingsModal(context, widget.appState);
  }

  void _openWifiSetupModal() {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (ctx) => Scaffold(
          appBar: AppBar(
            title: const Text('Thiết lập kết nối Wi-Fi'),
            leading: IconButton(
              icon: const Icon(Icons.arrow_back),
              onPressed: () => Navigator.of(ctx).pop(),
            ),
          ),
          body: WifiSetupScreen(
            appState: widget.appState,
          ),
        ),
      ),
    );
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
        onOpenWifi: _openWifiSetupModal,
        onOpenSettings: _openSettingsModal,
      ),
      body: Stack(
        children: [
          PageView(
            controller: _pageController,
            physics: const NeverScrollableScrollPhysics(),
            onPageChanged: (index) {
              setState(() {
                _currentIndex = index;
              });
            },
            children: screens,
          ),

          // Pairing PIN Dialog Overlay (Requirements 5)
          if (state.pendingPinCode != null && state.pendingPinCode!.isNotEmpty)
            Container(
              color: Colors.black54,
              alignment: Alignment.center,
              child: PairingPinDialog(
                pinCode: state.pendingPinCode!,
                deviceName: state.pendingPinDeviceName ?? '',
                deviceMac: state.pendingPinDeviceMac ?? '',
                onConfirm: () => state.dismissPinDialog(),
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
