import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet, Alert } from 'react-native';
import { Card } from '../common/Card';
import { useApp } from '../../context/AppContext';
import { colors } from '../../theme/colors';
import { typography } from '../../theme/typography';

export const TripSummaryStats: React.FC = () => {
  const { trips, clearTripHistory } = useApp();

  const totalDist = trips.reduce((acc, t) => acc + t.distanceKm, 0);
  const maxSpeed = trips.reduce((acc, t) => Math.max(acc, t.maxSpeedKmh), 0);

  const confirmClear = () => {
    Alert.alert('Xóa lịch sử', 'Bạn có chắc muốn xóa tất cả chuyến đi?', [
      { text: 'Hủy', style: 'cancel' },
      { text: 'Xóa', style: 'destructive', onPress: clearTripHistory },
    ]);
  };

  return (
    <Card
      label="TỔNG HỢP HÀNH TRÌNH"
      badge={
        trips.length > 0 ? (
          <TouchableOpacity onPress={confirmClear}>
            <Text style={styles.clearText}>Xóa lịch sử</Text>
          </TouchableOpacity>
        ) : undefined
      }
    >
      <View style={styles.grid}>
        <View style={styles.statBox}>
          <Text style={styles.num}>{trips.length}</Text>
          <Text style={styles.label}>Chuyến đi</Text>
        </View>

        <View style={styles.statBox}>
          <Text style={styles.num}>{totalDist.toFixed(1)}</Text>
          <Text style={styles.label}>Tổng km</Text>
        </View>

        <View style={styles.statBox}>
          <Text style={styles.num}>{maxSpeed}</Text>
          <Text style={styles.label}>Tối đa km/h</Text>
        </View>
      </View>
    </Card>
  );
};

const styles = StyleSheet.create({
  clearText: {
    fontSize: 11,
    fontWeight: '700',
    color: colors.accentRose,
  },
  grid: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: 4,
  },
  statBox: {
    flex: 1,
    alignItems: 'center',
  },
  num: {
    ...typography.metricMedium,
    color: colors.textPrimary,
  },
  label: {
    fontSize: 11,
    color: colors.textSecondary,
    marginTop: 2,
  },
});
