import { TransportClient } from './transport';
import { NativeICarMeter } from '../native/ICarMeterNative';
import { WsClientCommand, WsServerMessage } from '../types/websocket';

export class BtSppTransport implements TransportClient {
  readonly type = 'bt-spp' as const;
  private isConnected = false;
  private piMac: string;
  private connListeners: Set<(connected: boolean) => void> = new Set();
  private msgListeners: Set<(msg: WsServerMessage) => void> = new Set();
  private connSub: any = null;
  private msgSub: any = null;
  private reconnectTimer: any = null;
  private manualStop = false;

  constructor(piMac = '') {
    this.piMac = piMac;
  }

  setPiMac(mac: string) {
    if (this.piMac !== mac) {
      this.piMac = mac;
      if (this.isConnected) {
        this.stop();
        this.start();
      }
    }
  }

  get connected(): boolean {
    return this.isConnected;
  }

  start(): void {
    this.manualStop = false;
    this.setupListeners();
    this.connect();
  }

  private setupListeners() {
    if (!this.connSub) {
      this.connSub = NativeICarMeter.onBtConnectionChange((event) => {
        this.setConnected(event.connected);
        if (event.connected) {
          // Connected via BT SPP: request initial config
          this.send({ type: 'get_config' });
        } else if (!this.manualStop) {
          this.scheduleReconnect();
        }
      });
    }

    if (!this.msgSub) {
      this.msgSub = NativeICarMeter.onBtMessage((event) => {
        try {
          const parsed = JSON.parse(event.message);
          this.msgListeners.forEach((l) => l(parsed));
        } catch (err) {
          console.warn('[BT-SPP] Failed to parse message line:', event.message);
        }
      });
    }
  }

  private async connect() {
    if (!this.piMac) {
      return;
    }
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    try {
      await NativeICarMeter.connectBtSpp(this.piMac);
    } catch {
      this.setConnected(false);
      if (!this.manualStop) {
        this.scheduleReconnect();
      }
    }
  }

  private scheduleReconnect() {
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    this.reconnectTimer = setTimeout(() => {
      if (!this.manualStop && !this.isConnected && this.piMac) {
        this.connect();
      }
    }, 4000);
  }

  stop(): void {
    this.manualStop = true;
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    NativeICarMeter.disconnectBtSpp();
    this.setConnected(false);
  }

  send(cmd: WsClientCommand): boolean {
    if (!this.isConnected) return false;
    try {
      NativeICarMeter.btSend(JSON.stringify(cmd));
      return true;
    } catch {
      return false;
    }
  }

  private setConnected(connected: boolean) {
    if (this.isConnected !== connected) {
      this.isConnected = connected;
      this.connListeners.forEach((l) => l(connected));
    }
  }

  onConnectionChange(cb: (connected: boolean) => void): () => void {
    this.connListeners.add(cb);
    cb(this.isConnected);
    return () => {
      this.connListeners.delete(cb);
    };
  }

  onMessage(cb: (msg: WsServerMessage) => void): () => void {
    this.msgListeners.add(cb);
    return () => {
      this.msgListeners.delete(cb);
    };
  }
}
