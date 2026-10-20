import React, { createContext, useContext, useEffect, useMemo, useRef, useState } from 'react';
import { AppLocalSettings, PiFullConfig, TpmsConfig } from '../types/config';
import { TelemetryState } from '../types/telemetry';
import { ActiveTripState, TripRecord } from '../types/trip';
import { AutoTestProgress } from '../services/bleTpmsAdvertiser';
import { WifiWsTransport } from '../api/wifiWsTransport';
import { BtSppTransport } from '../api/btSppTransport';
import { PiApiService } from '../api/piApi';
import { StorageService } from '../services/storage';
import { TripManager } from '../services/tripManager';
import { GpsTracker } from '../services/gpsTracker';
import { BleTpmsAutoTester } from '../services/bleTpmsAdvertiser';
import { NativeICarMeter } from '../native/ICarMeterNative';

interface AppContextValue {
  isBtConnected: boolean;
  isWifiConnected: boolean;
  settings: AppLocalSettings;
  config: PiFullConfig;
  syncTimestamp: string;
  telemetry: TelemetryState;
  activeTrip: ActiveTripState;
  trips: TripRecord[];
  autoTest: AutoTestProgress;
  toastMessage: string | null;
  wifiTransport: WifiWsTransport;
  btTransport: BtSppTransport;

  setPiHost: (host: string) => Promise<void>;
  setPhoneMac: (mac: string) => Promise<void>;
  setPiBtMac: (mac: string) => Promise<void>;
  setSendRateHz: (hz: number) => Promise<void>;
  setBackgroundServiceEnabled: (enabled: boolean) => Promise<void>;
  updateTpmsField: (field: keyof TpmsConfig, value: any) => void;
  updateGpsHardwareToggle: (enabled: boolean) => Promise<void>;
  saveAndSyncConfig: (overrideConfig?: TpmsConfig) => Promise<boolean>;
  reloadConfigFromPi: () => Promise<boolean>;
  connectBt: (mac: string) => Promise<boolean>;
  disconnectBt: () => Promise<boolean>;
  startTrip: () => void;
  stopTrip: () => Promise<void>;
  clearTripHistory: () => Promise<void>;
  startAutoTest: (pressure: number, temp: number) => Promise<void>;
  stopAutoTest: () => Promise<void>;
  forceReconnectWifi: () => void;
  showToast: (msg: string) => void;
}

const AppContext = createContext<AppContextValue | null>(null);

export const AppProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [settings, setSettings] = useState<AppLocalSettings>({
    piHost: '192.168.4.1:8080',
    phoneMac: '',
    piBtMac: '',
    sendRateHz: 5,
    backgroundServiceEnabled: true,
  });

  const [config, setConfig] = useState<PiFullConfig>({
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
  });

  const [isBtConnected, setIsBtConnected] = useState(false);
  const [isWifiConnected, setIsWifiConnected] = useState(false);
  const [syncTimestamp, setSyncTimestamp] = useState('Chờ kết nối...');
  const [trips, setTrips] = useState<TripRecord[]>([]);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const [telemetry, setTelemetry] = useState<TelemetryState>({
    liveSpeedKmh: 0,
    accuracyM: null,
    altitudeM: null,
    sendRateHz: 5,
    sendCount: 0,
    lastSendTime: 0,
  });

  const [activeTrip, setActiveTrip] = useState<ActiveTripState>({
    isRecording: false,
    startTime: null,
    distanceKm: 0,
    maxSpeedKmh: 0,
    avgSpeedKmh: 0,
    durationStr: '00:00',
    speedSamples: [],
  });

  const [autoTest, setAutoTest] = useState<AutoTestProgress>({
    isRunning: false,
    stepTitle: 'Sẵn sàng',
    stepDesc: 'Nhấn Bắt Đầu để kiểm tra 4 van cảm biến',
    percent: 0,
    statusBadge: 'SẴN SÀNG',
  });

  // Singleton transport and service instances
  const wifiTransportRef = useRef<WifiWsTransport>(new WifiWsTransport(settings.piHost));
  const btTransportRef = useRef<BtSppTransport>(new BtSppTransport(settings.piBtMac || ''));
  const tripManagerRef = useRef<TripManager>(new TripManager());
  const autoTesterRef = useRef<BleTpmsAutoTester>(new BleTpmsAutoTester());

  const configRef = useRef(config);
  configRef.current = config;
  const settingsRef = useRef(settings);
  settingsRef.current = settings;

  const gpsTrackerRef = useRef<GpsTracker>(
    new GpsTracker(
      btTransportRef.current,
      tripManagerRef.current,
      () => configRef.current.gps.enabled
    )
  );

  const showToast = (msg: string) => {
    setToastMessage(msg);
  };

  useEffect(() => {
    if (toastMessage) {
      const timer = setTimeout(() => setToastMessage(null), 3000);
      return () => clearTimeout(timer);
    }
  }, [toastMessage]);

  // Initial Load & Service Setup
  useEffect(() => {
    async function init() {
      // 1. Load stored settings & cached config & trips
      const [storedSettings, cachedConfig, storedTrips, autoMac] = await Promise.all([
        StorageService.loadSettings(),
        StorageService.loadCachedConfig(),
        StorageService.loadTrips(),
        NativeICarMeter.getDeviceBluetoothAddress(),
      ]);

      const effectiveMac = storedSettings.phoneMac || autoMac;
      const finalSettings = { ...storedSettings, phoneMac: effectiveMac };
      if (!storedSettings.phoneMac && autoMac) {
        await StorageService.savePhoneMac(autoMac);
      }

      setSettings(finalSettings);
      setConfig(cachedConfig);
      setTrips(storedTrips);

      wifiTransportRef.current.setHost(storedSettings.piHost);
      btTransportRef.current.setPiMac(storedSettings.piBtMac || '');
      gpsTrackerRef.current.setSendRate(storedSettings.sendRateHz);

      if (storedSettings.backgroundServiceEnabled) {
        NativeICarMeter.startForegroundService();
      }

      // 2. Start Tracker & TripManager subscriptions
      tripManagerRef.current.subscribe((ts) => setActiveTrip(ts));
      gpsTrackerRef.current.subscribe((tel) => setTelemetry(tel));
      autoTesterRef.current.subscribe((prog) => setAutoTest(prog));

      gpsTrackerRef.current.start();

      // 3. Setup Bluetooth RFCOMM transport listeners
      btTransportRef.current.onConnectionChange((connected) => {
        setIsBtConnected(connected);
        if (connected) {
          const nowStr = new Date().toLocaleTimeString('vi-VN', { hour: '2-digit', minute: '2-digit' });
          setSyncTimestamp(`Đã kết nối Bluetooth (${nowStr})`);
        } else {
          setSyncTimestamp('Chờ kết nối Bluetooth...');
        }
      });

      btTransportRef.current.onMessage((msg) => {
        if (msg.type === 'welcome' && msg.config) {
          applyServerConfig(msg.config);
        } else if (msg.type === 'config' && msg.data) {
          applyServerConfig(msg.data);
          if (msg.message === 'Saved') {
            showToast('Đã lưu cấu hình');
          }
        } else if (msg.type === 'error') {
          showToast(`Lỗi: ${msg.message || 'Không thể lưu cấu hình'}`);
        }
      });

      // 4. Setup WiFi WebSocket listeners (used only during pairing setup)
      wifiTransportRef.current.onConnectionChange((connected) => {
        setIsWifiConnected(connected);
      });

      // If Pi Bluetooth MAC already saved, start BT connection
      if (storedSettings.piBtMac) {
        btTransportRef.current.start();
      }
    }

    init();

    return () => {
      gpsTrackerRef.current.stop();
      btTransportRef.current.stop();
      wifiTransportRef.current.stop();
    };
  }, []);

  const applyServerConfig = (serverConfig: PiFullConfig) => {
    if (!serverConfig) return;
    const merged: PiFullConfig = {
      tpms: {
        fl: serverConfig.tpms?.fl || '',
        fr: serverConfig.tpms?.fr || '',
        rl: serverConfig.tpms?.rl || '',
        rr: serverConfig.tpms?.rr || '',
        alert: typeof serverConfig.tpms?.alert === 'number' ? serverConfig.tpms.alert : 2.2,
        high_temp_alert: typeof serverConfig.tpms?.high_temp_alert === 'number' ? serverConfig.tpms.high_temp_alert : 70.0,
      },
      gps: {
        enabled: typeof serverConfig.gps?.enabled === 'boolean' ? serverConfig.gps.enabled : true,
      },
    };

    setConfig(merged);
    StorageService.saveCachedConfig(merged);
    const nowStr = new Date().toLocaleTimeString('vi-VN', { hour: '2-digit', minute: '2-digit' });
    setSyncTimestamp(`Đồng bộ (${nowStr})`);
  };

  const setPiHost = async (host: string) => {
    await StorageService.savePiHost(host);
    setSettings((prev) => ({ ...prev, piHost: host }));
    wifiTransportRef.current.setHost(host);
  };

  const setPhoneMac = async (mac: string) => {
    await StorageService.savePhoneMac(mac);
    setSettings((prev) => ({ ...prev, phoneMac: mac }));
  };

  const setPiBtMac = async (mac: string) => {
    await StorageService.savePiBtMac(mac);
    setSettings((prev) => ({ ...prev, piBtMac: mac }));
    btTransportRef.current.setPiMac(mac);
  };

  const setSendRateHz = async (hz: number) => {
    await StorageService.saveGpsRate(hz);
    setSettings((prev) => ({ ...prev, sendRateHz: hz }));
    gpsTrackerRef.current.setSendRate(hz);
  };

  const setBackgroundServiceEnabled = async (enabled: boolean) => {
    await StorageService.saveBackgroundService(enabled);
    setSettings((prev) => ({ ...prev, backgroundServiceEnabled: enabled }));
    if (enabled) {
      NativeICarMeter.startForegroundService();
      showToast('Bật chạy ngầm');
    } else {
      NativeICarMeter.stopForegroundService();
      showToast('Tắt chạy ngầm');
    }
  };

  const updateTpmsField = (field: keyof TpmsConfig, value: any) => {
    setConfig((prev) => ({
      ...prev,
      tpms: {
        ...prev.tpms,
        [field]: value,
      },
    }));
  };

  const updateGpsHardwareToggle = async (enabled: boolean) => {
    const nextConfig = {
      ...config,
      gps: { enabled },
    };
    setConfig(nextConfig);
    PiApiService.saveConfig(btTransportRef.current, nextConfig.tpms, enabled);
    StorageService.saveCachedConfig(nextConfig);
    showToast(enabled ? 'Dùng GPS xe' : 'Dùng GPS điện thoại');
  };

  const saveAndSyncConfig = async (overrideTpms?: TpmsConfig): Promise<boolean> => {
    const targetTpms = overrideTpms || config.tpms;
    const ok = PiApiService.saveConfig(
      btTransportRef.current,
      targetTpms,
      config.gps.enabled
    );

    if (ok) {
      const nowStr = new Date().toLocaleTimeString('vi-VN', { hour: '2-digit', minute: '2-digit' });
      setSyncTimestamp(`Đang gửi (${nowStr})`);
    } else {
      showToast('Bluetooth chưa kết nối');
    }
    return ok;
  };

  const reloadConfigFromPi = async (): Promise<boolean> => {
    const ok = PiApiService.fetchConfig(btTransportRef.current);
    if (ok) {
      showToast('Đang tải cấu hình...');
      return true;
    }
    showToast('Bluetooth chưa kết nối');
    return false;
  };

  const connectBt = async (mac: string): Promise<boolean> => {
    try {
      await setPiBtMac(mac);
      btTransportRef.current.setPiMac(mac);
      btTransportRef.current.start();
      return true;
    } catch (e: any) {
      showToast(`Lỗi kết nối: ${e?.message || 'Không thể kết nối'}`);
      return false;
    }
  };

  const disconnectBt = async (): Promise<boolean> => {
    btTransportRef.current.stop();
    return true;
  };

  const startTrip = () => {
    tripManagerRef.current.startTrip();
    showToast('Bắt đầu chuyến đi');
  };

  const stopTrip = async () => {
    const saved = await tripManagerRef.current.stopTrip();
    if (saved) {
      setTrips((prev) => [saved, ...prev]);
      showToast(`Đã lưu: ${saved.distanceKm} km (${saved.durationMin}p)`);
    }
  };

  const clearTripHistory = async () => {
    await StorageService.clearTrips();
    setTrips([]);
    showToast('Đã xóa lịch sử');
  };

  const startAutoTest = async (pressure: number, temp: number) => {
    const mac = settings.phoneMac.trim();
    if (!mac) {
      showToast('Chưa có MAC Bluetooth');
      return;
    }

    autoTesterRef.current.runAutoTest(
      mac,
      pressure,
      temp,
      config.tpms,
      async (stepConfig) => {
        return PiApiService.saveConfig(
          btTransportRef.current,
          stepConfig,
          config.gps.enabled
        );
      }
    );
  };

  const stopAutoTest = async () => {
    await autoTesterRef.current.stopAutoTest(async (restored) => {
      return PiApiService.saveConfig(
        btTransportRef.current,
        restored,
        config.gps.enabled
      );
    });
    showToast('Đã dừng test');
  };

  const forceReconnectWifi = () => {
    showToast('Đang kết nối lại WiFi...');
    NativeICarMeter.bindToCarWifi();
    wifiTransportRef.current.start();
  };

  const value: AppContextValue = useMemo(
    () => ({
      isBtConnected,
      isWifiConnected,
      settings,
      config,
      syncTimestamp,
      telemetry,
      activeTrip,
      trips,
      autoTest,
      toastMessage,
      wifiTransport: wifiTransportRef.current,
      btTransport: btTransportRef.current,
      setPiHost,
      setPhoneMac,
      setPiBtMac,
      setSendRateHz,
      setBackgroundServiceEnabled,
      updateTpmsField,
      updateGpsHardwareToggle,
      saveAndSyncConfig,
      reloadConfigFromPi,
      connectBt,
      disconnectBt,
      startTrip,
      stopTrip,
      clearTripHistory,
      startAutoTest,
      stopAutoTest,
      forceReconnectWifi,
      showToast,
    }),
    [
      isBtConnected,
      isWifiConnected,
      settings,
      config,
      syncTimestamp,
      telemetry,
      activeTrip,
      trips,
      autoTest,
      toastMessage,
    ]
  );

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
};

export const useApp = () => {
  const context = useContext(AppContext);
  if (!context) {
    throw new Error('useApp must be used within an AppProvider');
  }
  return context;
};
