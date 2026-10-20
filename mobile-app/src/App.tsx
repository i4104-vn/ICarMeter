import React, { useEffect, useState } from 'react';
import { SafeAreaView, StatusBar, View, StyleSheet, ActivityIndicator, Text } from 'react-native';
import { AppProvider, useApp } from './context/AppContext';
import { AppHeader } from './components/common/AppHeader';
import { Toast } from './components/common/Toast';
import { PillNavBar, TabId } from './navigation/PillNavBar';
import { DashboardScreen } from './screens/DashboardScreen';
import { TpmsScreen } from './screens/TpmsScreen';
import { TripsScreen } from './screens/TripsScreen';
import { WifiSetupScreen } from './screens/WifiSetupScreen';
import { BtPairingScreen } from './screens/BtPairingScreen';
import { BtLogsScreen } from './screens/BtLogsScreen';
import { colors } from './theme/colors';
import { NativeICarMeter } from './native/ICarMeterNative';

type AppFlow = 'checking' | 'wifi_setup' | 'bt_pairing' | 'main';

const AppContent: React.FC = () => {
  const [appFlow, setAppFlow] = useState<AppFlow>('checking');
  const [activeTab, setActiveTab] = useState<TabId>('dashboard');
  const { settings, connectBt, setPiBtMac } = useApp();

  useEffect(() => {
    async function checkPairedStatus() {
      try {
        const paired = await NativeICarMeter.getPhonePairedDevices();
        const pi = paired.find(
          (d) =>
            (settings.piBtMac && d.mac.toUpperCase() === settings.piBtMac.toUpperCase()) ||
            d.name.toLowerCase().includes('icarmeter')
        );

        if (pi) {
          await setPiBtMac(pi.mac);
          await connectBt(pi.mac);
          setAppFlow('main');
        } else {
          setAppFlow('wifi_setup');
        }
      } catch {
        setAppFlow('wifi_setup');
      }
    }

    checkPairedStatus();
  }, []);

  if (appFlow === 'checking') {
    return (
      <View style={styles.loadingContainer}>
        <ActivityIndicator size="large" color={colors.accentCyan} />
        <Text style={styles.loadingText}>Đang kiểm tra kết nối...</Text>
      </View>
    );
  }

  if (appFlow === 'wifi_setup') {
    return <WifiSetupScreen onWifiReady={() => setAppFlow('bt_pairing')} />;
  }

  if (appFlow === 'bt_pairing') {
    return (
      <BtPairingScreen
        onPairSuccess={async (piMac) => {
          if (piMac) {
            await connectBt(piMac);
          }
          setAppFlow('main');
        }}
        onBackToWifi={() => setAppFlow('wifi_setup')}
      />
    );
  }

  const renderScreen = () => {
    switch (activeTab) {
      case 'tpms':
        return <TpmsScreen />;
      case 'trips':
        return <TripsScreen />;
      case 'bt_logs':
        return <BtLogsScreen />;
      case 'dashboard':
      default:
        return <DashboardScreen />;
    }
  };

  return (
    <View style={styles.screenContainer}>
      <AppHeader onOpenPairing={() => setAppFlow('wifi_setup')} />
      <View style={styles.contentWrap}>{renderScreen()}</View>
      <PillNavBar activeTab={activeTab} onTabChange={setActiveTab} />
      <Toast />
    </View>
  );
};

export default function App() {
  return (
    <SafeAreaView style={styles.safeArea}>
      <StatusBar barStyle="light-content" backgroundColor={colors.bgPrimary} />
      <AppProvider>
        <AppContent />
      </AppProvider>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safeArea: {
    flex: 1,
    backgroundColor: colors.bgPrimary,
  },
  screenContainer: {
    flex: 1,
    backgroundColor: colors.bgPrimary,
  },
  contentWrap: {
    flex: 1,
  },
  loadingContainer: {
    flex: 1,
    backgroundColor: colors.bgPrimary,
    justifyContent: 'center',
    alignItems: 'center',
  },
  loadingText: {
    marginTop: 12,
    color: colors.textSecondary,
    fontSize: 13,
  },
});
