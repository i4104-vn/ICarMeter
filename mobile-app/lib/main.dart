import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'theme/app_theme.dart';
import 'services/storage_service.dart';
import 'services/app_state.dart';
import 'widgets/app_header.dart';
import 'widgets/pill_nav_bar.dart';
import 'widgets/pairing_pin_dialog.dart';
import 'screens/dashboard_screen.dart';
import 'screens/tpms_screen.dart';
import 'screens/trips_screen.dart';
import 'screens/bt_pairing_screen.dart';
import 'screens/bt_logs_screen.dart';
import 'screens/wifi_setup_screen.dart';
import 'screens/settings_screen.dart';

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

  void _onSelectTab(int index) {
    setState(() {
      _currentIndex = index;
    });
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
            onNavigateToBt: () {
              Navigator.of(ctx).pop();
              _onSelectTab(3); // Tab 3 is Bluetooth
            },
          ),
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final state = widget.appState;

    final screens = [
      DashboardScreen(
        appState: state,
        onNavigateToBt: () => _onSelectTab(3),
        onNavigateToWifi: _openWifiSetupModal,
        onNavigateToTpms: () => _onSelectTab(1),
      ),
      TpmsScreen(appState: state),
      TripsScreen(appState: state),
      BtPairingScreen(appState: state),
      BtLogsScreen(appState: state),
      SettingsScreen(appState: state),
    ];

    return Scaffold(
      appBar: AppHeader(
        appState: state,
        onOpenBluetooth: () => _onSelectTab(3),
        onOpenWifi: _openWifiSetupModal,
      ),
      body: Stack(
        children: [
          IndexedStack(
            index: _currentIndex,
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
      ),
    );
  }
}
