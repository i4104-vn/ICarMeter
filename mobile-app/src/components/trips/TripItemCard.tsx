import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { TripRecord } from '../../types/trip';
import { colors } from '../../theme/colors';

interface TripItemCardProps {
  trip: TripRecord;
}

export const TripItemCard: React.FC<TripItemCardProps> = ({ trip }) => {
  return (
    <View style={styles.card}>
      <View style={styles.topRow}>
        <Text style={styles.date}>{trip.dateStr}</Text>
        <Text style={styles.distance}>{trip.distanceKm} km</Text>
      </View>
      <View style={styles.bottomRow}>
        <Text style={styles.detail}>⏱ {trip.durationMin} phút</Text>
        <Text style={styles.detail}>⚡ TB {trip.avgSpeedKmh} km/h</Text>
        <Text style={styles.detail}>🚀 Max {trip.maxSpeedKmh} km/h</Text>
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  card: {
    backgroundColor: colors.bgCard,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: 12,
    padding: 12,
    marginBottom: 10,
  },
  topRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 8,
  },
  date: {
    fontSize: 13,
    fontWeight: '700',
    color: colors.textPrimary,
  },
  distance: {
    fontSize: 15,
    fontWeight: '800',
    color: colors.accentCyan,
  },
  bottomRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  detail: {
    fontSize: 11,
    color: colors.textSecondary,
  },
});
