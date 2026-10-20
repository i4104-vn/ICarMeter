import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { Card } from '../common/Card';
import { useApp } from '../../context/AppContext';
import { colors } from '../../theme/colors';
import { typography } from '../../theme/typography';

export const SpeedometerCard: React.FC = () => {
  const { telemetry, config } = useApp();

  return (
    <Card label="TỐC ĐỘ GPS">
      <View style={styles.speedDisplay}>
        <Text style={styles.speedNumber}>{telemetry.liveSpeedKmh}</Text>
        <Text style={styles.speedUnit}>KM/H</Text>
      </View>

      <View style={styles.telemetryGrid}>
        <View style={styles.telemetryItem}>
          <Text style={styles.telLabel}>Độ chính xác</Text>
          <Text style={styles.telVal}>
            {telemetry.accuracyM !== null ? `${telemetry.accuracyM} m` : '-- m'}
          </Text>
        </View>

        <View style={styles.telemetryItem}>
          <Text style={styles.telLabel}>Độ cao</Text>
          <Text style={styles.telVal}>
            {telemetry.altitudeM !== null ? `${telemetry.altitudeM} m` : '-- m'}
          </Text>
        </View>

        <View style={styles.telemetryItem}>
          <Text style={styles.telLabel}>Tần số gửi</Text>
          <Text style={styles.telVal}>
            {config.gps.enabled ? 'Hw GPS' : `${telemetry.sendRateHz} Hz`}
          </Text>
        </View>
      </View>
    </Card>
  );
};

const styles = StyleSheet.create({
  speedDisplay: {
    alignItems: 'center',
    justifyContent: 'center',
    paddingVertical: 12,
  },
  speedNumber: {
    ...typography.metricLarge,
    color: colors.textPrimary,
  },
  speedUnit: {
    fontSize: 14,
    fontWeight: '800',
    color: colors.accentCyan,
    letterSpacing: 2,
    marginTop: -4,
  },
  telemetryGrid: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginTop: 16,
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: colors.borderSubtle,
  },
  telemetryItem: {
    flex: 1,
    alignItems: 'center',
  },
  telLabel: {
    fontSize: 11,
    color: colors.textSecondary,
    marginBottom: 4,
  },
  telVal: {
    fontSize: 14,
    fontWeight: '700',
    color: colors.textPrimary,
  },
});
