/**
 * ICarMeter Companion App - Wi-Fi Architecture
 * Single Source of Truth: All configs are stored on Raspberry Pi.
 * Mobile communicates over Wi-Fi Hotspot (192.168.4.1:8080) to read, sync,
 * and test TPMS / GPS configurations in real time.
 *
 * Connection reliability: aggressive auto-reconnect with exponential backoff,
 * request queuing during disconnection, and visual connection state feedback.
 */

const defaultHost = (typeof window !== 'undefined' && window.location && window.location.host && !window.location.protocol.startsWith('file'))
    ? window.location.host
    : '192.168.4.1:8080';

// App State
const state = {
    piHost: localStorage.getItem('icarmeter_pi_host') || defaultHost,
    phoneMac: localStorage.getItem('icarmeter_phone_mac') || '',
    wifiConnected: false,

    // Synced State from Pi (Single Source of Truth)
    isSyncedWithPi: false,
    config: {
        tpms: {
            fl: "",
            fr: "",
            rl: "",
            rr: "",
            alert: 2.2,
            high_temp_alert: 70.0
        },
        gps: {
            enabled: true
        }
    },
    get piConfig() { return this.config; },
    get Config() { return this.config; },

    // Telemetry & GPS Forwarding
    watchId: null,
    lastPosition: null,
    liveSpeedKmh: 0,
    sendRateHz: 5,
    sendCount: 0,
    lastSendTime: 0,

    // Live Trip Recording
    isRecordingTrip: false,
    tripStartTime: null,
    tripDistanceKm: 0.0,
    tripMaxSpeed: 0.0,
    tripSpeedSamples: [],
    tripInterval: null,

    trips: []
};

// ==============================================================================
// WebSocket Connection Engine — Real-time Bi-directional Stream
// ==============================================================================
const wsConn = {
    socket: null,
    reconnectTimer: null,
    pingInterval: null,
    backoffMs: 1000,
    MIN_BACKOFF: 1000,
    MAX_BACKOFF: 6000,
    isConnected: false,
    _manualClose: false,

    start() {
        this.connect();
    },

    connect() {
        if (this.socket && (this.socket.readyState === WebSocket.CONNECTING || this.socket.readyState === WebSocket.OPEN)) {
            return;
        }

        if (this.socket) {
            try { this.socket.close(); } catch (_) {}
            this.socket = null;
        }

        this._manualClose = false;
        const host = state.piHost || defaultHost;
        const wsUrl = `ws://${host}/ws`;
        console.log(`[WS] Connecting to ${wsUrl}...`);

        try {
            this.socket = new WebSocket(wsUrl);
        } catch (err) {
            console.warn("[WS] Failed to instantiate WebSocket:", err);
            this.onDisconnected();
            this.scheduleReconnect();
            return;
        }

        this.socket.onopen = () => {
            console.log("[WS] Connected successfully!");
            this.isConnected = true;
            this.backoffMs = this.MIN_BACKOFF;
            updateWifiStatusUI(true);

            // Start heartbeat ping
            if (this.pingInterval) clearInterval(this.pingInterval);
            this.pingInterval = setInterval(() => {
                if (this.socket && this.socket.readyState === WebSocket.OPEN) {
                    this.send({ type: 'ping' });
                }
            }, 4000);

            // Request config on open
            this.send({ type: 'get_config' });
        };

        this.socket.onmessage = (event) => {
            try {
                const msg = JSON.parse(event.data);
                this.handleMessage(msg);
            } catch (e) {
                console.warn("[WS] Failed to parse message:", event.data);
            }
        };

        this.socket.onerror = () => {
            // onclose will handle reconnection
        };

        this.socket.onclose = () => {
            console.log("[WS] Disconnected");
            this.onDisconnected();
            if (!this._manualClose) {
                this.scheduleReconnect();
            }
        };
    },

    onDisconnected() {
        this.isConnected = false;
        updateWifiStatusUI(false);
        if (this.pingInterval) {
            clearInterval(this.pingInterval);
            this.pingInterval = null;
        }
    },

    scheduleReconnect() {
        if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
        this.reconnectTimer = setTimeout(() => {
            this.connect();
        }, this.backoffMs);
        this.backoffMs = Math.min(this.backoffMs * 1.5, this.MAX_BACKOFF);
    },

    forceReconnect() {
        console.log("[WS] Force reconnecting...");
        this.backoffMs = this.MIN_BACKOFF;
        if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
        if (this.socket) {
            try {
                this._manualClose = true;
                this.socket.close();
            } catch (_) {}
            this.socket = null;
        }
        this.connect();
    },

    send(obj) {
        if (this.socket && this.socket.readyState === WebSocket.OPEN) {
            this.socket.send(JSON.stringify(obj));
            return true;
        }
        return false;
    },

    handleMessage(msg) {
        const type = msg.type;

        if (type === 'welcome') {
            updateWifiStatusUI(true);
            if (msg.config) {
                applyConfigToStateAndUI(msg.config);
            }
        } else if (type === 'config') {
            if (msg.data) {
                applyConfigToStateAndUI(msg.data);
            }
            if (msg.message === 'Saved') {
                showToast("Đã lưu & đồng bộ cấu hình vào Pi!");
            }
        } else if (type === 'status' || type === 'heartbeat') {
            updateWifiStatusUI(true);
        } else if (type === 'pong') {
            updateWifiStatusUI(true);
        }
    }
};

const connMgr = wsConn;
window.connMgr = wsConn;

/**
 * Universal API Fetch with robust timeout and retry
 * Uses native Android HTTP bridge when running in Android Companion APK
 * (bypasses Android WebView cleartext blocks, CORS restrictions, and cellular 4G/5G routing conflicts).
 * Falls back to standard fetch in browsers.
 */
async function apiFetch(url, options = {}) {
    const method = (options.method || 'GET').toUpperCase();
    const bodyStr = options.body ? (typeof options.body === 'string' ? options.body : JSON.stringify(options.body)) : '';
    const timeoutMs = options._timeout || 3500;

    // Try native Android bridge first (bypasses Android cleartext / captive portal issues)
    if (window.AndroidBleTpms && typeof window.AndroidBleTpms.httpGet === 'function') {
        try {
            const resJsonStr = method === 'POST'
                ? window.AndroidBleTpms.httpPost(url, bodyStr)
                : window.AndroidBleTpms.httpGet(url);

            const res = JSON.parse(resJsonStr);
            if (res.status >= 200 && res.status < 300) {
                return {
                    ok: true,
                    status: res.status,
                    json: async () => JSON.parse(res.body || '{}'),
                    text: async () => res.body || ''
                };
            } else if (res.status === 0) {
                // Bridge returned a network error — fall through to fetch
                throw new Error(res.error || 'Bridge network error');
            } else {
                throw new Error(res.error || `HTTP ${res.status}`);
            }
        } catch (bridgeErr) {
            // If bridge explicitly reports network error, still try fetch as fallback
            if (bridgeErr.message && bridgeErr.message.includes('Bridge network error')) {
                // fall through to browser fetch
            } else {
                throw bridgeErr;
            }
        }
    }

    // Browser fetch with AbortController timeout
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), timeoutMs);
    try {
        const fetchOpts = { ...options, signal: controller.signal };
        delete fetchOpts._timeout;
        const res = await fetch(url, fetchOpts);
        clearTimeout(timeoutId);
        return res;
    } catch (err) {
        clearTimeout(timeoutId);
        throw err;
    }
}

/**
 * Resilient API call with automatic retry for transient failures.
 * Used for important operations (config sync, telemetry push).
 */
async function apiFetchWithRetry(url, options = {}, maxRetries = 2) {
    let lastErr;
    for (let attempt = 0; attempt <= maxRetries; attempt++) {
        try {
            const res = await apiFetch(url, options);
            if (res.ok) return res;
            // Non-OK status — don't retry client errors (4xx)
            if (res.status >= 400 && res.status < 500) return res;
            lastErr = new Error(`HTTP ${res.status}`);
        } catch (err) {
            lastErr = err;
        }
        // Wait before retry: 500ms, 1000ms
        if (attempt < maxRetries) {
            await new Promise(r => setTimeout(r, 500 * (attempt + 1)));
        }
    }
    throw lastErr;
}

// ==============================================================================
// App Lifecycle & Initialization
// ==============================================================================
document.addEventListener('DOMContentLoaded', () => {
    loadLocalFallbackSettings();
    initWifiCompanionUI();
    renderTripHistory();
    startGpsTracker();

    // Start the resilient connection manager
    connMgr.start();
});

function loadLocalFallbackSettings() {
    try {
        const savedRate = localStorage.getItem('icarmeter_gps_rate');
        if (savedRate) {
            state.sendRateHz = parseInt(savedRate, 10) || 5;
            const sel = document.getElementById('selectGpsRate');
            if (sel) sel.value = state.sendRateHz.toString();
        }

        const savedTrips = localStorage.getItem('icarmeter_trips');
        if (savedTrips) {
            state.trips = JSON.parse(savedTrips);
        }

        const savedHost = localStorage.getItem('icarmeter_pi_host');
        if (savedHost) {
            state.piHost = savedHost;
            const hostInput = document.getElementById('inputPiHost');
            if (hostInput) hostInput.value = savedHost;
        }

        const savedMac = localStorage.getItem('icarmeter_phone_mac');
        if (savedMac) {
            state.phoneMac = savedMac;
            const macInput = document.getElementById('inputTestPhoneMac');
            if (macInput) macInput.value = savedMac;
        }
    } catch (e) {
        console.warn("Storage load error:", e);
    }
}

function initWifiCompanionUI() {
    const hostInput = document.getElementById('inputPiHost');
    if (hostInput && !hostInput.value) {
        hostInput.value = state.piHost;
    }
    const macInput = document.getElementById('inputTestPhoneMac');
    if (macInput && !macInput.value && state.phoneMac) {
        macInput.value = state.phoneMac;
    }
    updateWifiStatusUI(false);

    // Make status badge tappable to force reconnect
    const badge = document.getElementById('wifiStatusBadge');
    if (badge) {
        badge.style.cursor = 'pointer';
        badge.addEventListener('click', () => {
            showToast("Đang kết nối lại...");
            connMgr.forceReconnect();
        });
    }
}

function savePiHost() {
    const hostInput = document.getElementById('inputPiHost');
    if (hostInput) {
        state.piHost = hostInput.value.trim() || '192.168.4.1:8080';
        localStorage.setItem('icarmeter_pi_host', state.piHost);
        connMgr.forceReconnect();
    }
}

function savePhoneMac() {
    const macInput = document.getElementById('inputTestPhoneMac');
    if (macInput) {
        state.phoneMac = macInput.value.trim().toUpperCase();
        localStorage.setItem('icarmeter_phone_mac', state.phoneMac);
    }
}

function saveSettings() {
    const sel = document.getElementById('selectGpsRate');
    if (sel) {
        state.sendRateHz = parseInt(sel.value, 10);
        localStorage.setItem('icarmeter_gps_rate', state.sendRateHz.toString());
    }
    showToast("Đã lưu thiết lập trên điện thoại");
}

// ==============================================================================
// Tab Navigation
// ==============================================================================
function switchTab(tabId) {
    document.querySelectorAll('.tab-pane').forEach(el => el.classList.remove('active'));
    document.querySelectorAll('.pill-nav-item').forEach(el => el.classList.remove('active'));

    const targetTab = document.getElementById(tabId);
    if (targetTab) targetTab.classList.add('active');

    const targetNav = document.querySelector(`.pill-nav-item[data-tab="${tabId}"]`);
    if (targetNav) targetNav.classList.add('active');
}

// ==============================================================================
// Wi-Fi Connection Status Badge
// ==============================================================================
function updateWifiStatusUI(connected) {
    const prev = state.wifiConnected;
    state.wifiConnected = connected;
    const badge = document.getElementById('wifiStatusBadge');
    const text = document.getElementById('wifiStatusText');

    if (!badge || !text) return;

    if (connected) {
        badge.classList.add('connected');
        text.textContent = "Đã kết nối";
        // Flash green briefly on reconnect
        if (!prev) {
            badge.style.transition = 'background-color 0.3s';
            badge.style.backgroundColor = 'rgba(16, 185, 129, 0.2)';
            setTimeout(() => { badge.style.backgroundColor = ''; }, 800);
        }
    } else {
        badge.classList.remove('connected');
        text.textContent = "Chưa kết nối";
    }
}

// Legacy compat — still callable from HTML onclick
async function checkAndAutoConnect(isManual = false) {
    if (isManual) {
        connMgr.forceReconnect();
    }
}

// Helper to update state and UI when configuration is received
function applyConfigToStateAndUI(data, isManual = false) {
    if (!data) return;
    if (data.tpms) {
        state.piConfig.tpms.fl = data.tpms.fl || "";
        state.piConfig.tpms.fr = data.tpms.fr || "";
        state.piConfig.tpms.rl = data.tpms.rl || "";
        state.piConfig.tpms.rr = data.tpms.rr || "";
        state.piConfig.tpms.alert = data.tpms.alert || data.tpms.low_pressure_alert || 2.2;
        state.piConfig.tpms.high_temp_alert = data.tpms.high_temp_alert || data.tpms.temp_alert || 70.0;

        const flInput = document.getElementById('inputFlSensor');
        const frInput = document.getElementById('inputFrSensor');
        const rlInput = document.getElementById('inputRlSensor');
        const rrInput = document.getElementById('inputRrSensor');
        const alertInput = document.getElementById('inputPressureAlert');
        const tempAlertInput = document.getElementById('inputTempAlert');

        if (flInput) flInput.value = state.piConfig.tpms.fl;
        if (frInput) frInput.value = state.piConfig.tpms.fr;
        if (rlInput) rlInput.value = state.piConfig.tpms.rl;
        if (rrInput) rrInput.value = state.piConfig.tpms.rr;
        if (alertInput) alertInput.value = state.piConfig.tpms.alert;
        if (tempAlertInput) tempAlertInput.value = state.piConfig.tpms.high_temp_alert;
    }

    if (data.gps) {
        state.piConfig.gps.enabled = !!data.gps.enabled;
        const toggle = document.getElementById('toggleHardwareGps');
        if (toggle) toggle.checked = state.piConfig.gps.enabled;
        updateGpsModeUI();
    }

    state.isSyncedWithPi = true;
    updateWifiStatusUI(true);

    const nowStr = new Date().toLocaleTimeString('vi-VN', { hour: '2-digit', minute: '2-digit' });
    const meta = document.getElementById('tpmsSyncTimestamp');
    if (meta) meta.textContent = `Đã đồng bộ (${nowStr})`;

    if (isManual) {
        showToast("Đã tải cấu hình từ xe thành công!");
    }
}

// ==============================================================================
// Config Operations (WebSocket-first with HTTP Fallback)
// ==============================================================================
async function fetchConfigFromPi(isManual = false) {
    if (wsConn.isConnected) {
        wsConn.send({ type: "get_config" });
        if (isManual) showToast("Đang tải cấu hình qua WebSocket...");
        return true;
    }

    // Fallback to HTTP API
    try {
        const res = await apiFetchWithRetry(`http://${state.piHost}/api/config`, {
            _timeout: 4000
        }, isManual ? 3 : 1);

        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        const data = await res.json();
        applyConfigToStateAndUI(data, isManual);
        return true;
    } catch (err) {
        updateWifiStatusUI(false);
        if (isManual) {
            showToast(`Không thể đọc cấu hình: ${err.message}`);
        }
        return false;
    }
}

async function saveAndSyncTpmsToPi(overrideConfig = null) {
    const fl = overrideConfig ? overrideConfig.fl : (document.getElementById('inputFlSensor')?.value.trim().toUpperCase() || "");
    const fr = overrideConfig ? overrideConfig.fr : (document.getElementById('inputFrSensor')?.value.trim().toUpperCase() || "");
    const rl = overrideConfig ? overrideConfig.rl : (document.getElementById('inputRlSensor')?.value.trim().toUpperCase() || "");
    const rr = overrideConfig ? overrideConfig.rr : (document.getElementById('inputRrSensor')?.value.trim().toUpperCase() || "");
    const alertPressure = overrideConfig ? overrideConfig.alert : (parseFloat(document.getElementById('inputPressureAlert')?.value) || 2.2);
    const alertTemp = overrideConfig ? (overrideConfig.high_temp_alert ?? overrideConfig.temp_alert ?? 70.0) : (parseFloat(document.getElementById('inputTempAlert')?.value) || 70.0);
    const gpsHwEnabled = document.getElementById('toggleHardwareGps')?.checked ?? true;

    const payload = {
        action: "update_config",
        fl: fl,
        fr: fr,
        rl: rl,
        rr: rr,
        alert: alertPressure,
        high_temp_alert: alertTemp,
        gps_enabled: gpsHwEnabled
    };

    // Update local state and UI immediately
    state.piConfig.tpms.fl = fl;
    state.piConfig.tpms.fr = fr;
    state.piConfig.tpms.rl = rl;
    state.piConfig.tpms.rr = rr;
    state.piConfig.tpms.alert = alertPressure;
    state.piConfig.tpms.high_temp_alert = alertTemp;
    state.piConfig.gps.enabled = gpsHwEnabled;
    state.isSyncedWithPi = true;

    const nowStr = new Date().toLocaleTimeString('vi-VN', { hour: '2-digit', minute: '2-digit' });
    const meta = document.getElementById('tpmsSyncTimestamp');
    if (meta && !overrideConfig) meta.textContent = `Đã lưu (${nowStr})`;

    // WebSocket-first transmission
    if (wsConn.isConnected) {
        wsConn.send({
            type: "set_config",
            payload: payload
        });
        if (!overrideConfig) {
            showToast("Đã gửi & đồng bộ cấu hình qua WebSocket!");
        }
        return true;
    }

    // Fallback: HTTP POST
    try {
        const res = await apiFetchWithRetry(`http://${state.piHost}/api/config`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
            _timeout: 4000
        }, 2);

        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        updateWifiStatusUI(true);

        if (!overrideConfig) {
            showToast("Đã lưu cấu hình thành công!");
        }
        return true;
    } catch (err) {
        showToast(`Lỗi khi ghi cấu hình: ${err.message}`);
        return false;
    }
}

// ==============================================================================
// Auto Test TPMS Workflow
// ==============================================================================
const autoTestState = {
    running: false,
    original: null
};

function delay(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

function updateAutoTestUI(title, desc, percent) {
    const titleEl = document.getElementById('autoTestStepTitle');
    const descEl = document.getElementById('autoTestStepDesc');
    const barEl = document.getElementById('autoTestProgressBar');

    if (titleEl) titleEl.textContent = title;
    if (descEl) descEl.textContent = desc;
    if (barEl) barEl.style.width = `${percent}%`;
}

async function startAutoTpmsTest() {
    const macInput = document.getElementById('inputTestPhoneMac');
    const phoneMac = (macInput?.value || '').trim().toUpperCase();

    if (!phoneMac || phoneMac.length < 8) {
        showToast("Vui lòng nhập địa chỉ Bluetooth MAC hợp lệ của điện thoại!");
        macInput?.focus();
        return;
    }

    savePhoneMac();

    // 1. Snapshot original sensor IDs & alert thresholds
    const original = {
        fl: document.getElementById('inputFlSensor')?.value.trim().toUpperCase() || state.piConfig.tpms.fl,
        fr: document.getElementById('inputFrSensor')?.value.trim().toUpperCase() || state.piConfig.tpms.fr,
        rl: document.getElementById('inputRlSensor')?.value.trim().toUpperCase() || state.piConfig.tpms.rl,
        rr: document.getElementById('inputRrSensor')?.value.trim().toUpperCase() || state.piConfig.tpms.rr,
        alert: parseFloat(document.getElementById('inputPressureAlert')?.value) || 2.2,
        high_temp_alert: parseFloat(document.getElementById('inputTempAlert')?.value) || 70.0
    };

    autoTestState.running = true;
    autoTestState.original = original;

    // Update UI elements to running state
    document.getElementById('btnStartAutoTest').style.display = 'none';
    document.getElementById('btnStopAutoTest').style.display = 'inline-block';
    document.getElementById('autoTestProgress').style.display = 'block';

    const badge = document.getElementById('autoTestStatusBadge');
    if (badge) {
        badge.textContent = "ĐANG TEST";
        badge.style.background = "#eab308";
        badge.style.color = "#000";
    }

    const testPressure = parseFloat(document.getElementById('inputTestPressure')?.value) || 2.4;
    const testTemp = parseFloat(document.getElementById('inputTestTemp')?.value) || 75.0;

    const testSteps = [
        { label: "Trước Trái (FL)", key: "fl", percent: 25 },
        { label: "Trước Phải (FR)", key: "fr", percent: 50 },
        { label: "Sau Trái (RL)", key: "rl", percent: 75 },
        { label: "Sau Phải (RR)", key: "rr", percent: 100 }
    ];

    for (let i = 0; i < testSteps.length; i++) {
        if (!autoTestState.running) break;

        const step = testSteps[i];

        // 1. Set tested wheel ID to phoneMac
        const testConfig = {
            fl: step.key === "fl" ? phoneMac : original.fl,
            fr: step.key === "fr" ? phoneMac : original.fr,
            rl: step.key === "rl" ? phoneMac : original.rl,
            rr: step.key === "rr" ? phoneMac : original.rr,
            alert: original.alert,
            high_temp_alert: original.high_temp_alert
        };

        // Update input fields in UI
        if (document.getElementById('inputFlSensor')) document.getElementById('inputFlSensor').value = testConfig.fl;
        if (document.getElementById('inputFrSensor')) document.getElementById('inputFrSensor').value = testConfig.fr;
        if (document.getElementById('inputRlSensor')) document.getElementById('inputRlSensor').value = testConfig.rl;
        if (document.getElementById('inputRrSensor')) document.getElementById('inputRrSensor').value = testConfig.rr;

        // Push to Pi over Wi-Fi
        await saveAndSyncTpmsToPi(testConfig);

        // 2. Transmit BLE TPMS packets over the air from mobile device
        if (window.AndroidBleTpms && typeof window.AndroidBleTpms.startBleAdvertising === 'function') {
            try {
                window.AndroidBleTpms.startBleAdvertising(phoneMac, testPressure, testTemp);
            } catch (bleErr) {
                console.warn("[BLE-TPMS] Native BLE advertiser start failed:", bleErr);
            }
        }

        // Countdown 5 seconds per wheel while phone transmits BLE packets
        for (let countdown = 5; countdown > 0; countdown--) {
            if (!autoTestState.running) break;
            updateAutoTestUI(
                `Bước ${i + 1}/4: Kiểm tra bánh ${step.label} (${countdown}s)`,
                `Đang phát sóng BLE từ điện thoại (${testPressure} bar, ${testTemp}°C) tới xe cho bánh ${step.label} [${phoneMac}]...`,
                step.percent
            );
            await delay(1000);
        }

        // Stop BLE advertising for this wheel before switching or ending
        if (window.AndroidBleTpms && typeof window.AndroidBleTpms.stopBleAdvertising === 'function') {
            try {
                window.AndroidBleTpms.stopBleAdvertising();
            } catch (_) {}
        }
    }

    // 3. Restore original IDs back to the Pi!
    await restoreOriginalTpmsConfig();
}

async function restoreOriginalTpmsConfig() {
    // Ensure native BLE advertising is stopped
    if (window.AndroidBleTpms && typeof window.AndroidBleTpms.stopBleAdvertising === 'function') {
        try {
            window.AndroidBleTpms.stopBleAdvertising();
        } catch (_) {}
    }

    if (!autoTestState.original) return;
    const original = autoTestState.original;

    updateAutoTestUI(
        "Đang khôi phục ID cảm biến ban đầu...",
        "Đang lưu lại cấu hình 4 van gốc...",
        100
    );

    if (document.getElementById('inputFlSensor')) document.getElementById('inputFlSensor').value = original.fl;
    if (document.getElementById('inputFrSensor')) document.getElementById('inputFrSensor').value = original.fr;
    if (document.getElementById('inputRlSensor')) document.getElementById('inputRlSensor').value = original.rl;
    if (document.getElementById('inputRrSensor')) document.getElementById('inputRrSensor').value = original.rr;
    if (document.getElementById('inputPressureAlert')) document.getElementById('inputPressureAlert').value = original.alert;
    if (document.getElementById('inputTempAlert')) document.getElementById('inputTempAlert').value = original.high_temp_alert;

    await saveAndSyncTpmsToPi(original);

    autoTestState.running = false;
    document.getElementById('btnStartAutoTest').style.display = 'inline-block';
    document.getElementById('btnStopAutoTest').style.display = 'none';

    const badge = document.getElementById('autoTestStatusBadge');
    if (badge) {
        badge.textContent = "HOÀN TẤT";
        badge.style.background = "#22c55e";
        badge.style.color = "#000";
    }

    updateAutoTestUI(
        "Hoàn tất Auto Test qua BLE!",
        "Đã phát sóng BLE kiểm tra cả 4 vị trí bánh và tự động khôi phục toàn bộ ID gốc ban đầu.",
        100
    );
    showToast("Auto Test qua BLE hoàn tất! Đã khôi phục các ID cảm biến ban đầu.");
}

function stopAutoTpmsTest() {
    autoTestState.running = false;
    if (window.AndroidBleTpms && typeof window.AndroidBleTpms.stopBleAdvertising === 'function') {
        try {
            window.AndroidBleTpms.stopBleAdvertising();
        } catch (_) {}
    }
    restoreOriginalTpmsConfig();
    showToast("Đã dừng Auto Test và khôi phục cấu hình gốc.");
}

// ==============================================================================
// GPS Tracking & BLE Forwarding (Tethering)
// ==============================================================================
function startGpsTracker() {
    if (!('geolocation' in navigator)) {
        showToast("Thiết bị không hỗ trợ định vị GPS.");
        return;
    }

    state.watchId = navigator.geolocation.watchPosition(
        onGpsPositionUpdate,
        onGpsError,
        {
            enableHighAccuracy: true,
            maximumAge: 0,
            timeout: 10000
        }
    );
}

function onGpsPositionUpdate(pos) {
    state.lastPosition = pos;
    const coords = pos.coords;

    // Convert m/s to km/h
    let speedKmh = 0;
    if (coords.speed !== null && !isNaN(coords.speed) && coords.speed > 0) {
        speedKmh = Math.round(coords.speed * 3.6);
    }
    state.liveSpeedKmh = speedKmh;

    // Update UI numbers
    document.getElementById('liveSpeedNumber').textContent = speedKmh;
    document.getElementById('telAccuracy').textContent = coords.accuracy ? `${Math.round(coords.accuracy)} m` : '-- m';
    document.getElementById('telAltitude').textContent = coords.altitude ? `${Math.round(coords.altitude)} m` : '-- m';

    // If trip recording is active, sample telemetry
    if (state.isRecordingTrip) {
        sampleTripTelemetry(speedKmh);
    }

    // Forward to over BLE (only if hardware GPS on is disabled)
    if (!state.piConfig.gps.enabled) {
        maybeTransmitGpsToPi(coords, speedKmh);
    }
}

function onGpsError(err) {
    console.warn("GPS error:", err.message);
}

/** Telemetry send failure counter — backs off after repeated failures */
let _gpsSendFailures = 0;

function maybeTransmitGpsToPi(coords, speedKmh) {
    const now = Date.now();
    const intervalMs = 1000 / state.sendRateHz;

    if (now - state.lastSendTime < intervalMs) {
        return; // Throttle to target Hz to preserve phone battery
    }

    state.lastSendTime = now;
    state.sendCount++;
    const rateEl = document.getElementById('telSendRate');
    if (rateEl) rateEl.textContent = `${state.sendRateHz} Hz`;

    const payload = {
        spd: speedKmh,
        lat: parseFloat(coords.latitude.toFixed(6)),
        lon: parseFloat(coords.longitude.toFixed(6)),
        alt: coords.altitude ? Math.round(coords.altitude) : 0,
        hdg: coords.heading ? Math.round(coords.heading) : 0
    };

    // WebSocket-first telemetry transmission (zero HTTP overhead, instant push)
    if (wsConn.isConnected) {
        wsConn.send({
            type: "telemetry",
            spd: payload.spd,
            lat: payload.lat,
            lon: payload.lon,
            alt: payload.alt,
            hdg: payload.hdg
        });
        _gpsSendFailures = 0;
        return;
    }

    // If Pi is not reachable, back off HTTP telemetry sends to avoid hammering
    if (_gpsSendFailures > 5) {
        if (now - state.lastSendTime < 3000) return;
    }

    // Fallback: Forward telemetry over Wi-Fi Hotspot HTTP
    const host = state.piHost || defaultHost;
    apiFetch(`http://${host}/api/gps`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
        keepalive: true,
        _timeout: 2000
    }).then(() => {
        _gpsSendFailures = 0;
    }).catch(() => {
        _gpsSendFailures++;
    });
}

// GPS Option Toggle: Hardware GPS vs Phone Wi-Fi GPS
function onGpsModeToggle(checkbox) {
    state.piConfig.gps.enabled = checkbox.checked;
    updateGpsModeUI();
    saveAndSyncTpmsToPi();
}

function updateGpsModeUI() {
    const badge = document.getElementById('gpsSourceBadge');
    const title = document.getElementById('gpsToggleTitle');
    const desc = document.getElementById('gpsToggleDesc');

    if (!badge) return;

    if (state.piConfig.gps.enabled) {
        badge.textContent = "MODULE GPS";
        badge.classList.add('hardware-gps');
        if (title) title.textContent = "Module GPS phần cứng";
        if (desc) desc.textContent = "Đang BẬT: Lấy tốc độ qua Module GPS.";
    } else {
        badge.textContent = "ĐIỆN THOẠI";
        badge.classList.remove('hardware-gps');
        if (title) title.textContent = "Module GPS phần cứng";
        if (desc) desc.textContent = "Đang TẮT: Truyền tốc độ từ điện thoại qua Wi-Fi.";
    }
}

// ==============================================================================
// Trip History & Logging (Recorded & Synced with)
// ==============================================================================
function toggleTripRecording() {
    if (state.isRecordingTrip) {
        stopTripRecording();
    } else {
        startTripRecording();
    }
}

function startTripRecording() {
    state.isRecordingTrip = true;
    state.tripStartTime = Date.now();
    state.tripDistanceKm = 0.0;
    state.tripMaxSpeed = 0.0;
    state.tripSpeedSamples = [];

    document.getElementById('tripRecordBadge').style.display = 'inline-block';
    const btn = document.getElementById('btnTripAction');
    btn.textContent = "Kết Thúc Chuyến Đi";
    btn.className = "btn btn-danger";

    state.tripInterval = setInterval(updateTripTimerUI, 1000);
    showToast("Bắt đầu ghi nhận hành trình mới");
}

function sampleTripTelemetry(speedKmh) {
    state.tripSpeedSamples.push(speedKmh);
    if (speedKmh > state.tripMaxSpeed) {
        state.tripMaxSpeed = speedKmh;
    }

    // Distance delta = (v / 3600) * 1s
    state.tripDistanceKm += (speedKmh / 3600.0) * 1.0;
    document.getElementById('currentTripDistance').textContent = state.tripDistanceKm.toFixed(1);

    const avg = state.tripSpeedSamples.reduce((a, b) => a + b, 0) / state.tripSpeedSamples.length;
    document.getElementById('currentTripAvgSpeed').textContent = Math.round(avg);
}

function updateTripTimerUI() {
    if (!state.tripStartTime) return;
    const elapsedSec = Math.floor((Date.now() - state.tripStartTime) / 1000);
    const m = Math.floor(elapsedSec / 60).toString().padStart(2, '0');
    const s = (elapsedSec % 60).toString().padStart(2, '0');
    document.getElementById('currentTripDuration').textContent = `${m}:${s}`;
}

function stopTripRecording() {
    clearInterval(state.tripInterval);
    state.isRecordingTrip = false;

    const durationSec = Math.floor((Date.now() - state.tripStartTime) / 1000);
    const avgSpeed = state.tripSpeedSamples.length > 0
        ? Math.round(state.tripSpeedSamples.reduce((a, b) => a + b, 0) / state.tripSpeedSamples.length)
        : 0;

    const newTrip = {
        id: Date.now(),
        dateStr: new Date().toLocaleDateString('vi-VN', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }),
        distanceKm: parseFloat(state.tripDistanceKm.toFixed(1)),
        durationMin: Math.max(1, Math.round(durationSec / 60)),
        avgSpeedKmh: avgSpeed,
        maxSpeedKmh: Math.round(state.tripMaxSpeed)
    };

    state.trips.unshift(newTrip);
    localStorage.setItem('icarmeter_trips', JSON.stringify(state.trips));

    // Reset UI
    document.getElementById('tripRecordBadge').style.display = 'none';
    const btn = document.getElementById('btnTripAction');
    btn.textContent = "Bắt Đầu Chuyến Đi";
    btn.className = "btn btn-success";

    renderTripHistory();
    showToast(`Đã lưu chuyến đi: ${newTrip.distanceKm} km (${newTrip.durationMin} phút)`);
}

function renderTripHistory() {
    const list = document.getElementById('tripListContainer');
    const empty = document.getElementById('emptyTripsState');

    if (!state.trips || state.trips.length === 0) {
        list.innerHTML = '';
        list.appendChild(empty);
        document.getElementById('totalTripsCount').textContent = '0';
        document.getElementById('totalDistanceCount').textContent = '0.0';
        document.getElementById('maxSpeedEver').textContent = '0';
        return;
    }

    let totalDist = 0;
    let maxSpeed = 0;

    list.innerHTML = '';
    state.trips.forEach(trip => {
        totalDist += trip.distanceKm;
        if (trip.maxSpeedKmh > maxSpeed) maxSpeed = trip.maxSpeedKmh;

        const card = document.createElement('div');
        card.className = 'trip-item-card';
        card.innerHTML = `
            <div class="trip-item-top">
                <span class="trip-item-date">${trip.dateStr}</span>
                <span class="trip-item-dist">${trip.distanceKm} km</span>
            </div>
            <div class="trip-item-details">
                <span>⏱ ${trip.durationMin} phút</span>
                <span>⚡ TB: ${trip.avgSpeedKmh} km/h</span>
                <span>🚀 Max: ${trip.maxSpeedKmh} km/h</span>
            </div>
        `;
        list.appendChild(card);
    });

    document.getElementById('totalTripsCount').textContent = state.trips.length;
    document.getElementById('totalDistanceCount').textContent = totalDist.toFixed(1);
    document.getElementById('maxSpeedEver').textContent = maxSpeed;
}

function clearTripHistory() {
    if (!confirm("Bạn có chắc chắn muốn xóa toàn bộ lịch sử các chuyến đi?")) return;
    state.trips = [];
    localStorage.removeItem('icarmeter_trips');
    renderTripHistory();
    showToast("Đã xóa toàn bộ lịch sử chuyến đi");
}

// ==============================================================================
// Utility Toast
// ==============================================================================
let toastTimer = null;
function showToast(msg) {
    const t = document.getElementById('appToast');
    if (!t) return;
    t.textContent = msg;
    t.classList.add('show');
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
        t.classList.remove('show');
    }, 2800);
}
