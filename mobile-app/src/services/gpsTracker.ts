import { NativeICarMeter, NativeLocationEvent } from '../native/ICarMeterNative';
import { GpsTelemetry, TelemetryState } from '../types/telemetry';
import { PiApiService } from '../api/piApi';
import { TransportClient } from '../api/transport';
import { TripManager } from './tripManager';

export type TelemetryListener = (telemetry: TelemetryState) => void;

export class GpsTracker {
  private state: TelemetryState = {
    liveSpeedKmh: 0,
    accuracyM: null,
    altitudeM: null,
    sendRateHz: 5,
    sendCount: 0,
    lastSendTime: 0,
  };

  private isStarted = false;
  private transport: TransportClient;
  private tripManager: TripManager;
  private isHardwareGpsEnabled: () => boolean;
  private listeners: Set<TelemetryListener> = new Set();
  private locationSub: any = null;

  constructor(
    transport: TransportClient,
    tripManager: TripManager,
    isHardwareGpsEnabled: () => boolean
  ) {
    this.transport = transport;
    this.tripManager = tripManager;
    this.isHardwareGpsEnabled = isHardwareGpsEnabled;
  }

  setSendRate(hz: number) {
    this.state.sendRateHz = hz;
    this.notify();
  }

  start() {
    if (this.isStarted) return;
    this.isStarted = true;

    // Start native GPS provider
    NativeICarMeter.startNativeGps(this.state.sendRateHz);

    // Subscribe to location events from native bridge
    this.locationSub = NativeICarMeter.onLocationUpdate((event: NativeLocationEvent) => {
      this.handleLocationUpdate(event);
    });
  }

  stop() {
    if (!this.isStarted) return;
    this.isStarted = false;

    NativeICarMeter.stopNativeGps();
    if (this.locationSub) {
      this.locationSub.remove();
      this.locationSub = null;
    }
  }

  private handleLocationUpdate(loc: NativeLocationEvent) {
    // Speed conversion: loc.speed in m/s -> km/h
    let speedKmh = 0;
    if (loc.speed > 0 && !isNaN(loc.speed)) {
      speedKmh = Math.round(loc.speed * 3.6);
    }

    this.state.liveSpeedKmh = speedKmh;
    this.state.accuracyM = loc.accuracy ? Math.round(loc.accuracy) : null;
    this.state.altitudeM = loc.altitude ? Math.round(loc.altitude) : null;

    // Feed to active trip recorder
    this.tripManager.sampleSpeed(speedKmh);

    // If hardware GPS on Pi is disabled, transmit mobile GPS to Pi
    if (!this.isHardwareGpsEnabled()) {
      this.maybeTransmitTelemetry(loc, speedKmh);
    }

    this.notify();
  }

  private maybeTransmitTelemetry(loc: NativeLocationEvent, speedKmh: number) {
    const now = Date.now();
    const intervalMs = 1000 / this.state.sendRateHz;

    if (now - this.state.lastSendTime < intervalMs) {
      return; // Rate limit to target Hz
    }

    this.state.lastSendTime = now;
    this.state.sendCount++;

    const payload: GpsTelemetry = {
      spd: speedKmh,
      lat: parseFloat(loc.latitude.toFixed(6)),
      lon: parseFloat(loc.longitude.toFixed(6)),
      alt: loc.altitude ? Math.round(loc.altitude) : 0,
      hdg: loc.heading ? Math.round(loc.heading) : 0,
    };

    PiApiService.sendTelemetry(this.transport, payload);
  }

  subscribe(listener: TelemetryListener): () => void {
    this.listeners.add(listener);
    listener({ ...this.state });
    return () => {
      this.listeners.delete(listener);
    };
  }

  private notify() {
    this.listeners.forEach((l) => l({ ...this.state }));
  }

  getState(): TelemetryState {
    return { ...this.state };
  }
}
