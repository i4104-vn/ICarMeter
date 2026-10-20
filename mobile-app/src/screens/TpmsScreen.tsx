import React from 'react';
import { ScrollView, Text, StyleSheet } from 'react-native';
import { Card } from '../components/common/Card';
import { CarTpmsDiagram } from '../components/tpms/CarTpmsDiagram';
import { ThresholdSettings } from '../components/tpms/ThresholdSettings';
import { AutoTestCard } from '../components/tpms/AutoTestCard';
import { useApp } from '../context/AppContext';
import { colors } from '../theme/colors';

export const TpmsScreen: React.FC = () => {
  const { syncTimestamp } = useApp();

  return (
    <ScrollView
      style={styles.container}
      contentContainerStyle={styles.content}
      showsVerticalScrollIndicator={false}
    >
      <Card
        label="CẤU HÌNH VAN CẢM BIẾN"
        badge={<Text style={styles.syncMeta}>{syncTimestamp}</Text>}
      >
        <CarTpmsDiagram />
        <ThresholdSettings />
      </Card>

      <AutoTestCard />
    </ScrollView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  content: {
    padding: 16,
    paddingBottom: 90,
  },
  syncMeta: {
    fontSize: 11,
    color: colors.textSecondary,
    fontWeight: '600',
  },
});
