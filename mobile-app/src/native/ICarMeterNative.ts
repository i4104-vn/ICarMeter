import { NativeModules, NativeEventEmitter, Platform } from 'react-native';

const { ICarMeterBridge } = NativeModules;

export interface NativeLocationEvent {
  speed: number;
  latitude: number;
  longitude: number;
  altitude: number;
  heading: number;
  accuracy: number;
}

export interface NativeWifiEvent {
  connected: boolean;
  ssid?: string;
}

class ICarMeterNativeService {
  private eventEmitter: NativeEventEmitter | null = null;

  constructor() {
    if (ICarMeterBridge && Platform.OS === 'android') {
      this.eventEmitter = new NativeEventEmitter(ICarMeterBridge);
    }
  }

  async isBleAdvertisingSupported(): Promise<boolean> {
    if (!ICarMeterBridge) return false;
    try {
      return await ICarMeterBridge.isBleAdvertisingSupported();
    } catch {
      return false;
    }
  }

  async getDeviceBluetoothAddress(): Promise<string> {
    if (!ICarMeterBridge) return '';
    try {
      return await ICarMeterBridge.getDeviceBluetoothAddress();
    } catch {
      return '';
    }
  }

  async startBleAdvertising(phoneMac: string, pressureBar: number, tempC: number): Promise<boolean> {
    if (!ICarMeterBridge) return false;
    try {
      return await ICarMeterBridge.startBleAdvertising(phoneMac, pressureBar, tempC);
    } catch {
      return false;
    }
  }

  async stopBleAdvertising(): Promise<void> {
    if (!ICarMeterBridge) return;
    try {
      await ICarMeterBridge.stopBleAdvertising();
    } catch {
      // Ignore
    }
  }

  async bindToCarWifi(): Promise<boolean> {
    if (!ICarMeterBridge) return false;
    try {
      return await ICarMeterBridge.bindToCarWifi();
    } catch {
      return false;
    }
  }

  async startForegroundService(): Promise<void> {
    if (!ICarMeterBridge) return;
    try {
      await ICarMeterBridge.startForegroundService();
    } catch {
      // Ignore
    }
  }

  async stopForegroundService(): Promise<void> {
    if (!ICarMeterBridge) return;
    try {
      await ICarMeterBridge.stopForegroundService();
    } catch {
      // Ignore
    }
  }

  async startNativeGps(rateHz: number): Promise<void> {
    if (!ICarMeterBridge) return;
    try {
      await ICarMeterBridge.startGps(rateHz);
    } catch {
      // Ignore
    }
  }

  async stopNativeGps(): Promise<void> {
    if (!ICarMeterBridge) return;
    try {
      await ICarMeterBridge.stopGps();
    } catch {
      // Ignore
    }
  }

  async nativeHttpGet(url: string): Promise<{ status: number; body?: string; error?: string }> {
    if (ICarMeterBridge && ICarMeterBridge.httpGet) {
      try {
        const jsonStr = await ICarMeterBridge.httpGet(url);
        return JSON.parse(jsonStr);
      } catch (err: any) {
        return { status: 0, error: err?.message || 'Bridge error' };
      }
    }
    return { status: 0, error: 'Bridge unavailable' };
  }

  async nativeHttpPost(url: string, bodyJson: string): Promise<{ status: number; body?: string; error?: string }> {
    if (ICarMeterBridge && ICarMeterBridge.httpPost) {
      try {
        const jsonStr = await ICarMeterBridge.httpPost(url, bodyJson);
        return JSON.parse(jsonStr);
      } catch (err: any) {
        return { status: 0, error: err?.message || 'Bridge error' };
      }
    }
    return { status: 0, error: 'Bridge unavailable' };
  }

  async getPhonePairedDevices(): Promise<Array<{ mac: string; name: string }>> {
    if (!ICarMeterBridge || !ICarMeterBridge.getPhonePairedDevices) return [];
    try {
      return await ICarMeterBridge.getPhonePairedDevices();
    } catch {
      return [];
    }
  }

  async connectBtSpp(mac: string, uuid?: string): Promise<boolean> {
    if (!ICarMeterBridge || !ICarMeterBridge.connectBtSpp) return false;
    try {
      return await ICarMeterBridge.connectBtSpp(mac, uuid || null);
    } catch (e: any) {
      throw new Error(e?.message || 'Bluetooth connection failed');
    }
  }

  async disconnectBtSpp(): Promise<boolean> {
    if (!ICarMeterBridge || !ICarMeterBridge.disconnectBtSpp) return false;
    try {
      return await ICarMeterBridge.disconnectBtSpp();
    } catch {
      return false;
    }
  }

  async btSend(jsonMessage: string): Promise<boolean> {
    if (!ICarMeterBridge || !ICarMeterBridge.btSend) return false;
    try {
      return await ICarMeterBridge.btSend(jsonMessage);
    } catch {
      return false;
    }
  }

  async isBtSppConnected(): Promise<boolean> {
    if (!ICarMeterBridge || !ICarMeterBridge.isBtSppConnected) return false;
    try {
      return await ICarMeterBridge.isBtSppConnected();
    } catch {
      return false;
    }
  }

  onBtMessage(listener: (event: { message: string }) => void) {
    if (!this.eventEmitter) return { remove: () => {} };
    return this.eventEmitter.addListener('onBtMessage', listener);
  }

  onBtConnectionChange(listener: (event: { connected: boolean; mac?: string; name?: string }) => void) {
    if (!this.eventEmitter) return { remove: () => {} };
    return this.eventEmitter.addListener('onBtConnectionChange', listener);
  }

  onLocationUpdate(listener: (event: NativeLocationEvent) => void) {
    if (!this.eventEmitter) return { remove: () => {} };
    const subscription = this.eventEmitter.addListener('onLocationUpdate', listener);
    return subscription;
  }

  onWifiStateChange(listener: (event: NativeWifiEvent) => void) {
    if (!this.eventEmitter) return { remove: () => {} };
    const subscription = this.eventEmitter.addListener('onWifiStateChange', listener);
    return subscription;
  }
}

export const NativeICarMeter = new ICarMeterNativeService();
