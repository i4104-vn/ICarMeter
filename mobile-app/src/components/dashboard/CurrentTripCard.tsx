import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { Card } from '../common/Card';
import { PrimaryButton } from '../common/PrimaryButton';
import { useApp } from '../../context/AppContext';
import { colors } from '../../theme/colors';
import { typography } from '../../theme/typography';

export const CurrentTripCard: React.FC = () => {
  const { activeTrip, startTrip, stopTrip } = useApp();

  return (
    <Card
      label="CHUYẾN ĐI"
      badge={
        activeTrip.isRecording ? (
          <View style={styles.recordingBadge}>
            <Text style={styles.recordingText}>ĐANG GHI</Text>
          </View>
        ) : undefined
      }
    >
      <View style={styles.grid}>
        <View style={styles.statBox}>
          <Text style={styles.statNum}>{activeTrip.distanceKm.toFixed(1)}</Text>
          <Text style={styles.statUnit}>km</Text>
          <Text style={styles.statDesc}>Quãng đường</Text>
        </View>

        <View style={styles.statBox}>
          <Text style={styles.statNum}>{activeTrip.durationStr}</Text>
          <Text style={styles.statUnit}>phút</Text>
          <Text style={styles.statDesc}>Thời gian</Text>
        </View>

        <View style={styles.statBox}>
          <Text style={styles.statNum}>{activeTrip.avgSpeedKmh}</Text>
          <Text style={styles.statUnit}>km/h</Text>
          <Text style={styles.statDesc}>Vận tốc TB</Text>
        </View>
      </View>

      <View style={styles.actionWrap}>
        {activeTrip.isRecording ? (
          <PrimaryButton
            title="Kết thúc"
            variant="danger"
            onPress={stopTrip}
          />
        ) : (
          <PrimaryButton
            title="Bắt đầu"
            variant="success"
            onPress={startTrip}
          />
        )}
      </View>
    </Card>
  );
};

const styles = StyleSheet.create({
  recordingBadge: {
    backgroundColor: 'rgba(239, 68, 68, 0.2)',
    paddingHorizontal: 8,
    paddingVertical: 3,
    borderRadius: 6,
    borderWidth: 1,
    borderColor: 'rgba(239, 68, 68, 0.4)',
  },
  recordingText: {
    color: colors.accentRose,
    fontSize: 10,
    fontWeight: '800',
  },
  grid: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 8,
  },
  statBox: {
    flex: 1,
    alignItems: 'center',
  },
  statNum: {
    ...typography.metricMedium,
    color: colors.textPrimary,
  },
  statUnit: {
    fontSize: 11,
    fontWeight: '600',
    color: colors.accentCyan,
    marginTop: -2,
  },
  statDesc: {
    fontSize: 11,
    color: colors.textSecondary,
    marginTop: 4,
  },
  actionWrap: {
    marginTop: 12,
  },
});
