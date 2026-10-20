import { WsClientCommand, WsServerMessage } from '../types/websocket';

export type WsConnectionListener = (connected: boolean) => void;
export type WsMessageListener = (message: WsServerMessage) => void;

export class WsClient {
  private socket: WebSocket | null = null;
  private host: string;
  private isConnected = false;
  private backoffMs = 1000;
  private readonly minBackoff = 1000;
  private readonly maxBackoff = 6000;
  private reconnectTimer: any = null;
  private pingTimer: any = null;
  private manualClose = false;

  private connListeners: Set<WsConnectionListener> = new Set();
  private msgListeners: Set<WsMessageListener> = new Set();

  constructor(initialHost = '192.168.4.1:8080') {
    this.host = initialHost;
  }

  setHost(host: string) {
    if (this.host !== host) {
      this.host = host;
      this.forceReconnect();
    }
  }

  start() {
    this.connect();
  }

  stop() {
    this.manualClose = true;
    this.cleanupTimers();
    if (this.socket) {
      try {
        this.socket.close();
      } catch {}
      this.socket = null;
    }
    this.setConnected(false);
  }

  forceReconnect() {
    this.manualClose = false;
    this.backoffMs = this.minBackoff;
    this.cleanupTimers();
    if (this.socket) {
      try {
        this.socket.close();
      } catch {}
      this.socket = null;
    }
    this.connect();
  }

  private connect() {
    if (this.socket && (this.socket.readyState === WebSocket.CONNECTING || this.socket.readyState === WebSocket.OPEN)) {
      return;
    }

    this.cleanupTimers();
    const cleanHost = this.host.trim() || '192.168.4.1:8080';
    const wsUrl = `ws://${cleanHost}/ws`;

    try {
      this.socket = new WebSocket(wsUrl);
    } catch (e) {
      console.warn('[WS] Failed to create WebSocket:', e);
      this.setConnected(false);
      this.scheduleReconnect();
      return;
    }

    this.socket.onopen = () => {
      this.setConnected(true);
      this.backoffMs = this.minBackoff;

      // Start ping heartbeat
      this.pingTimer = setInterval(() => {
        if (this.socket && this.socket.readyState === WebSocket.OPEN) {
          this.send({ type: 'ping' });
        }
      }, 4000);

      // Request current configuration on connection
      this.send({ type: 'get_config' });
    };

    this.socket.onmessage = (event) => {
      try {
        const parsed = JSON.parse(event.data);
        this.msgListeners.forEach((l) => l(parsed));
      } catch (err) {
        console.warn('[WS] Parse error:', event.data);
      }
    };

    this.socket.onerror = () => {
      // onclose will handle reconnect
    };

    this.socket.onclose = () => {
      this.setConnected(false);
      this.cleanupTimers();
      if (!this.manualClose) {
        this.scheduleReconnect();
      }
    };
  }

  private setConnected(connected: boolean) {
    if (this.isConnected !== connected) {
      this.isConnected = connected;
      this.connListeners.forEach((l) => l(connected));
    }
  }

  private cleanupTimers() {
    if (this.pingTimer) {
      clearInterval(this.pingTimer);
      this.pingTimer = null;
    }
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }

  private scheduleReconnect() {
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    this.reconnectTimer = setTimeout(() => {
      this.connect();
    }, this.backoffMs);
    this.backoffMs = Math.min(this.backoffMs * 1.5, this.maxBackoff);
  }

  send(cmd: WsClientCommand): boolean {
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      try {
        this.socket.send(JSON.stringify(cmd));
        return true;
      } catch {
        return false;
      }
    }
    return false;
  }

  get connected(): boolean {
    return this.isConnected;
  }

  onConnectionChange(listener: WsConnectionListener): () => void {
    this.connListeners.add(listener);
    listener(this.isConnected);
    return () => this.connListeners.delete(listener);
  }

  onMessage(listener: WsMessageListener): () => void {
    this.msgListeners.add(listener);
    return () => this.msgListeners.delete(listener);
  }
}
