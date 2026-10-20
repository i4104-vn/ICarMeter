import { ActiveTripState, TripRecord } from '../types/trip';
import { StorageService } from './storage';

export class TripManager {
  private state: ActiveTripState = {
    isRecording: false,
    startTime: null,
    distanceKm: 0.0,
    maxSpeedKmh: 0,
    avgSpeedKmh: 0,
    durationStr: '00:00',
    speedSamples: [],
  };

  private timer: any = null;
  private onStateChange: ((state: ActiveTripState) => void) | null = null;

  subscribe(listener: (state: ActiveTripState) => void): () => void {
    this.onStateChange = listener;
    listener({ ...this.state });
    return () => {
      this.onStateChange = null;
    };
  }

  startTrip() {
    if (this.state.isRecording) return;

    this.state = {
      isRecording: true,
      startTime: Date.now(),
      distanceKm: 0.0,
      maxSpeedKmh: 0,
      avgSpeedKmh: 0,
      durationStr: '00:00',
      speedSamples: [],
    };

    if (this.timer) clearInterval(this.timer);
    this.timer = setInterval(() => {
      this.updateTimer();
    }, 1000);

    this.notify();
  }

  sampleSpeed(speedKmh: number) {
    if (!this.state.isRecording) return;

    const samples = [...this.state.speedSamples, speedKmh];
    const maxSpeedKmh = Math.max(this.state.maxSpeedKmh, speedKmh);
    const sum = samples.reduce((a, b) => a + b, 0);
    const avgSpeedKmh = Math.round(sum / samples.length);

    // Delta distance for 1 second interval: (speed km/h / 3600)
    const deltaKm = speedKmh / 3600.0;
    const distanceKm = parseFloat((this.state.distanceKm + deltaKm).toFixed(2));

    this.state = {
      ...this.state,
      distanceKm,
      maxSpeedKmh,
      avgSpeedKmh,
      speedSamples: samples,
    };

    this.notify();
  }

  private updateTimer() {
    if (!this.state.startTime) return;
    const elapsedSec = Math.floor((Date.now() - this.state.startTime) / 1000);
    const m = Math.floor(elapsedSec / 60)
      .toString()
      .padStart(2, '0');
    const s = (elapsedSec % 60).toString().padStart(2, '0');

    this.state = {
      ...this.state,
      durationStr: `${m}:${s}`,
    };

    this.notify();
  }

  async stopTrip(): Promise<TripRecord | null> {
    if (!this.state.isRecording || !this.state.startTime) return null;

    if (this.timer) {
      clearInterval(this.timer);
      this.timer = null;
    }

    const durationSec = Math.floor((Date.now() - this.state.startTime) / 1000);
    const durationMin = Math.max(1, Math.round(durationSec / 60));

    const dateStr = new Date().toLocaleDateString('vi-VN', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });

    const newTrip: TripRecord = {
      id: Date.now(),
      dateStr,
      distanceKm: parseFloat(this.state.distanceKm.toFixed(1)),
      durationMin,
      avgSpeedKmh: this.state.avgSpeedKmh,
      maxSpeedKmh: this.state.maxSpeedKmh,
    };

    // Save to storage
    const currentTrips = await StorageService.loadTrips();
    const updated = [newTrip, ...currentTrips];
    await StorageService.saveTrips(updated);

    this.state = {
      isRecording: false,
      startTime: null,
      distanceKm: 0.0,
      maxSpeedKmh: 0,
      avgSpeedKmh: 0,
      durationStr: '00:00',
      speedSamples: [],
    };

    this.notify();
    return newTrip;
  }

  private notify() {
    if (this.onStateChange) {
      this.onStateChange({ ...this.state });
    }
  }

  getState(): ActiveTripState {
    return { ...this.state };
  }
}
