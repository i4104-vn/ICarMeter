import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { colors } from '../../theme/colors';

export const EmptyTripsState: React.FC = () => {
  return (
    <View style={styles.container}>
      <Text style={styles.icon}>🚗</Text>
      <Text style={styles.text}>Chưa có chuyến đi nào được lưu.</Text>
      <Text style={styles.subtext}>Bấm "Bắt đầu" ở màn hình chính khi khởi hành.</Text>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    alignItems: 'center',
    justifyContent: 'center',
    paddingVertical: 36,
    paddingHorizontal: 20,
    backgroundColor: colors.bgCard,
    borderRadius: 14,
    borderWidth: 1,
    borderColor: colors.borderSubtle,
    borderStyle: 'dashed',
  },
  icon: {
    fontSize: 32,
    marginBottom: 8,
  },
  text: {
    fontSize: 13,
    fontWeight: '600',
    color: colors.textPrimary,
    textAlign: 'center',
  },
  subtext: {
    fontSize: 12,
    color: colors.textSecondary,
    textAlign: 'center',
    marginTop: 4,
  },
});
