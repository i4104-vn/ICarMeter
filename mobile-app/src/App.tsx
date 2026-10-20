import React, { useState } from 'react';
import { SafeAreaView, StatusBar, View, StyleSheet } from 'react-native';
import { AppProvider } from './context/AppContext';
import { AppHeader } from './components/common/AppHeader';
import { Toast } from './components/common/Toast';
import { PillNavBar, TabId } from './navigation/PillNavBar';
import { DashboardScreen } from './screens/DashboardScreen';
import { TpmsScreen } from './screens/TpmsScreen';
import { TripsScreen } from './screens/TripsScreen';
import { colors } from './theme/colors';

const MainNavigator: React.FC = () => {
  const [activeTab, setActiveTab] = useState<TabId>('dashboard');

  const renderScreen = () => {
    switch (activeTab) {
      case 'tpms':
        return <TpmsScreen />;
      case 'trips':
        return <TripsScreen />;
      case 'dashboard':
      default:
        return <DashboardScreen />;
    }
  };

  return (
    <View style={styles.screenContainer}>
      <AppHeader />
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
        <MainNavigator />
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
});
