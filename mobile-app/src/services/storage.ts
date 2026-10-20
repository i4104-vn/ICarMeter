import AsyncStorage from '@react-native-async-storage/async-storage';
import { PiFullConfig, AppLocalSettings } from '../types/config';
import { TripRecord } from '../types/trip';

const KEYS = {
  PI_HOST: 'icarmeter_pi_host',
  PHONE_MAC: 'icarmeter_phone_mac',
  GPS_RATE: 'icarmeter_gps_rate',
  BG_SERVICE: 'icarmeter_bg_service',
  TRIPS: 'icarmeter_trips',
  CONFIG_CACHE: 'icarmeter_config_cache',
};

const DEFAULT_SETTINGS: AppLocalSettings = {
  piHost: '192.168.4.1:8080',
  phoneMac: '',
  sendRateHz: 5,
  backgroundServiceEnabled: true,
};

const DEFAULT_CONFIG: PiFullConfig = {
  tpms: {
    fl: '',
    fr: '',
    rl: '',
    rr: '',
    alert: 2.2,
    high_temp_alert: 70.0,
  },
  gps: {
    enabled: true,
  },
};

export const StorageService = {
  async loadSettings(): Promise<AppLocalSettings> {
    try {
      const [host, mac, rate, bg] = await Promise.all([
        AsyncStorage.getItem(KEYS.PI_HOST),
        AsyncStorage.getItem(KEYS.PHONE_MAC),
        AsyncStorage.getItem(KEYS.GPS_RATE),
        AsyncStorage.getItem(KEYS.BG_SERVICE),
      ]);

      return {
        piHost: host || DEFAULT_SETTINGS.piHost,
        phoneMac: mac || DEFAULT_SETTINGS.phoneMac,
        sendRateHz: rate ? parseInt(rate, 10) || 5 : 5,
        backgroundServiceEnabled: bg !== null ? bg === 'true' : true,
      };
    } catch {
      return DEFAULT_SETTINGS;
    }
  },

  async savePiHost(host: string): Promise<void> {
    try {
      await AsyncStorage.setItem(KEYS.PI_HOST, host);
    } catch (e) {
      console.warn('Failed to save piHost', e);
    }
  },

  async savePhoneMac(mac: string): Promise<void> {
    try {
      await AsyncStorage.setItem(KEYS.PHONE_MAC, mac);
    } catch (e) {
      console.warn('Failed to save phoneMac', e);
    }
  },

  async saveGpsRate(rateHz: number): Promise<void> {
    try {
      await AsyncStorage.setItem(KEYS.GPS_RATE, rateHz.toString());
    } catch (e) {
      console.warn('Failed to save gpsRate', e);
    }
  },

  async saveBackgroundService(enabled: boolean): Promise<void> {
    try {
      await AsyncStorage.setItem(KEYS.BG_SERVICE, enabled ? 'true' : 'false');
    } catch (e) {
      console.warn('Failed to save backgroundService', e);
    }
  },

  async loadCachedConfig(): Promise<PiFullConfig> {
    try {
      const cached = await AsyncStorage.getItem(KEYS.CONFIG_CACHE);
      if (cached) {
        return JSON.parse(cached);
      }
    } catch {
      // Ignore
    }
    return DEFAULT_CONFIG;
  },

  async saveCachedConfig(config: PiFullConfig): Promise<void> {
    try {
      await AsyncStorage.setItem(KEYS.CONFIG_CACHE, JSON.stringify(config));
    } catch (e) {
      console.warn('Failed to cache config', e);
    }
  },

  async loadTrips(): Promise<TripRecord[]> {
    try {
      const raw = await AsyncStorage.getItem(KEYS.TRIPS);
      if (raw) {
        return JSON.parse(raw);
      }
    } catch {
      // Ignore
    }
    return [];
  },

  async saveTrips(trips: TripRecord[]): Promise<void> {
    try {
      await AsyncStorage.setItem(KEYS.TRIPS, JSON.stringify(trips));
    } catch (e) {
      console.warn('Failed to save trips', e);
    }
  },

  async clearTrips(): Promise<void> {
    try {
      await AsyncStorage.removeItem(KEYS.TRIPS);
    } catch (e) {
      console.warn('Failed to clear trips', e);
    }
  },
};
