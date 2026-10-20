import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';
import { colors } from '../theme/colors';

export type TabId = 'dashboard' | 'tpms' | 'trips';

interface PillNavBarProps {
  activeTab: TabId;
  onTabChange: (tab: TabId) => void;
}

const TABS: { id: TabId; label: string; icon: string }[] = [
  { id: 'dashboard', label: 'Đồng hồ', icon: '⏱' },
  { id: 'tpms', label: 'TPMS', icon: '🔘' },
  { id: 'trips', label: 'Hành trình', icon: '📋' },
];

export const PillNavBar: React.FC<PillNavBarProps> = ({ activeTab, onTabChange }) => {
  return (
    <View style={styles.navContainer}>
      <View style={styles.pill}>
        {TABS.map((tab) => {
          const isActive = activeTab === tab.id;
          return (
            <TouchableOpacity
              key={tab.id}
              activeOpacity={0.7}
              onPress={() => onTabChange(tab.id)}
              style={[styles.navItem, isActive && styles.navItemActive]}
            >
              <Text style={styles.icon}>{tab.icon}</Text>
              <Text style={[styles.label, isActive && styles.labelActive]}>
                {tab.label}
              </Text>
            </TouchableOpacity>
          );
        })}
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  navContainer: {
    position: 'absolute',
    bottom: 16,
    left: 20,
    right: 20,
    alignItems: 'center',
    zIndex: 900,
  },
  pill: {
    flexDirection: 'row',
    backgroundColor: '#111c34',
    borderRadius: 30,
    padding: 4,
    borderWidth: 1,
    borderColor: colors.borderActive,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 6 },
    shadowOpacity: 0.4,
    shadowRadius: 10,
    elevation: 10,
  },
  navItem: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
    paddingVertical: 8,
    paddingHorizontal: 16,
    borderRadius: 24,
  },
  navItemActive: {
    backgroundColor: 'rgba(56, 189, 248, 0.18)',
  },
  icon: {
    fontSize: 14,
  },
  label: {
    fontSize: 12,
    fontWeight: '600',
    color: colors.textSecondary,
  },
  labelActive: {
    color: colors.accentCyan,
    fontWeight: '700',
  },
});
