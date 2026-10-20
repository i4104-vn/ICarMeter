export interface BtDevice {
  mac: string;
  name: string;
  rssi?: number;
}

export interface BtStatusData {
  adapter_available: boolean;
  paired_devices: BtDevice[];
  rfcomm_listening: boolean;
}
