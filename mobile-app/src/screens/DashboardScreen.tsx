import React from 'react';
import { ScrollView, StyleSheet } from 'react-native';
import { GpsModeCard } from '../components/dashboard/GpsModeCard';
import { SpeedometerCard } from '../components/dashboard/SpeedometerCard';
import { CurrentTripCard } from '../components/dashboard/CurrentTripCard';
import { BackgroundSettingsCard } from '../components/dashboard/BackgroundSettingsCard';

export const DashboardScreen: React.FC = () => {
  return (
    <ScrollView
      style={styles.container}
      contentContainerStyle={styles.content}
      showsVerticalScrollIndicator={false}
    >
      <GpsModeCard />
      <SpeedometerCard />
      <CurrentTripCard />
      <BackgroundSettingsCard />
    </ScrollView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  content: {
    padding: 16,
    paddingBottom: 90, // Leave room for floating bottom navbar
  },
});
