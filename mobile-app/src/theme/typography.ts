import { TextStyle } from 'react-native';

export const typography: Record<string, TextStyle> = {
  h1: {
    fontSize: 22,
    fontWeight: '800',
    letterSpacing: 0.5,
  },
  h2: {
    fontSize: 17,
    fontWeight: '700',
    letterSpacing: 0.3,
  },
  label: {
    fontSize: 11,
    fontWeight: '700',
    letterSpacing: 1.2,
    textTransform: 'uppercase',
  },
  body: {
    fontSize: 14,
    fontWeight: '400',
  },
  bodySmall: {
    fontSize: 12,
    fontWeight: '400',
  },
  metricLarge: {
    fontSize: 64,
    fontWeight: '900',
    fontVariant: ['tabular-nums'],
    letterSpacing: -1,
  },
  metricMedium: {
    fontSize: 26,
    fontWeight: '800',
    fontVariant: ['tabular-nums'],
  },
};
