import React from 'react';
import { View, Text, StyleSheet, ViewStyle } from 'react-native';
import { colors } from '../../theme/colors';
import { typography } from '../../theme/typography';

interface CardProps {
  label?: string;
  badge?: React.ReactNode;
  children: React.ReactNode;
  style?: ViewStyle;
}

export const Card: React.FC<CardProps> = ({ label, badge, children, style }) => {
  return (
    <View style={[styles.card, style]}>
      {(label || badge) && (
        <View style={styles.cardHeader}>
          {label ? <Text style={styles.cardLabel}>{label}</Text> : <View />}
          {badge}
        </View>
      )}
      {children}
    </View>
  );
};

const styles = StyleSheet.create({
  card: {
    backgroundColor: colors.bgCard,
    borderRadius: 16,
    padding: 16,
    marginBottom: 14,
    borderWidth: 1,
    borderColor: colors.border,
  },
  cardHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  cardLabel: {
    ...typography.label,
    color: colors.textSecondary,
  },
});
