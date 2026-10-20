export interface TripRecord {
  id: number;
  dateStr: string;
  distanceKm: number;
  durationMin: number;
  avgSpeedKmh: number;
  maxSpeedKmh: number;
}

export interface ActiveTripState {
  isRecording: boolean;
  startTime: number | null;
  distanceKm: number;
  maxSpeedKmh: number;
  avgSpeedKmh: number;
  durationStr: string;
  speedSamples: number[];
}
