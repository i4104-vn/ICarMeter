import React from 'react';
import { View, Text, FlatList, StyleSheet } from 'react-native';
import { TripSummaryStats } from '../components/trips/TripSummaryStats';
import { TripItemCard } from '../components/trips/TripItemCard';
import { EmptyTripsState } from '../components/trips/EmptyTripsState';
import { useApp } from '../context/AppContext';
import { colors } from '../theme/colors';

export const TripsScreen: React.FC = () => {
  const { trips } = useApp();

  return (
    <View style={styles.container}>
      <FlatList
        data={trips}
        keyExtractor={(item) => item.id.toString()}
        ListHeaderComponent={
          <>
            <TripSummaryStats />
            <Text style={styles.sectionTitle}>Danh sách chuyến đi</Text>
          </>
        }
        renderItem={({ item }) => <TripItemCard trip={item} />}
        ListEmptyComponent={<EmptyTripsState />}
        contentContainerStyle={styles.content}
        showsVerticalScrollIndicator={false}
      />
    </View>
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
  sectionTitle: {
    fontSize: 13,
    fontWeight: '700',
    color: colors.textSecondary,
    marginBottom: 10,
    marginTop: 4,
    textTransform: 'uppercase',
    letterSpacing: 0.5,
  },
});
