/**
 * ICarMeter Companion App - Pure Flat Architecture
 * Single Source of Truth: All configs are stored on Raspberry Pi.
 * Mobile only reads, displays, and synchronizes back to Pi over Bluetooth LE.
 */

// ICarMeter Custom BLE GATT UUIDs
const BLE_SERVICE_UUID      = "1c480001-0000-1000-8000-00805f9b34fb";
const CHAR_TPMS_CONFIG_UUID = "1c480002-0000-1000-8000-00805f9b34fb"; // Read / Write Config
const CHAR_GPS_FEED_UUID    = "1c480003-0000-1000-8000-00805f9b34fb"; // Write GPS speed/coords
const CHAR_SYSTEM_CTRL_UUID = "1c480004-0000-1000-8000-00805f9b34fb"; // Read / Write System status
const CHAR_TRIP_STATS_UUID  = "1c480005-0000-1000-8000-00805f9b34fb"; // Read / Write Trips

// App State
const state = {
    connectedDevice: null,
    gattServer: null,
    charTpmsConfig: null,
    charGpsFeed: null,
    charSystemCtrl: null,
    charTripStats: null,
    
    // Synced State from Pi (Single Source of Truth)
    isSyncedWithPi: false,
    lastSyncTime: null,
    piConfig: {
        tpms: {
            fl: "",
            fr: "",
            rl: "",
            rr: "",
            alert: 2.2
        },
        gps: {
            enabled: false // false = Phone BLE GPS, true = Pi hardware GPS module
        }
    },
    
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
// App Lifecycle & Initialization
// ==============================================================================
document.addEventListener('DOMContentLoaded', () => {
    loadLocalFallbackSettings();
    updateSyncBarUI(false, "Nguồn dữ liệu: Đang chờ kết nối ICarMeter trên xe...");
    renderTripHistory();
    startGpsTracker();
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
    } catch (e) {
        console.warn("Storage load error:", e);
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
// Tab Navigation (Flat Pill Navbar)
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
// Bluetooth Low Energy (BLE) Client & Auto-Sync
// ==============================================================================
async function toggleBleConnection() {
    if (state.gattServer && state.gattServer.connected) {
        disconnectBle();
    } else {
        await connectBle();
    }
}

async function connectBle() {
    if (!navigator.bluetooth) {
        showToast("Trình duyệt không hỗ trợ Web Bluetooth. Vui lòng bật Bluetooth trên Android.");
        return;
    }

    try {
        updateBleStatusUI("connecting", "Đang quét xe...");
        updateSyncBarUI(false, "Đang quét thiết bị ICarMeter...");

        // Scan for ICarMeter device
        const device = await navigator.bluetooth.requestDevice({
            filters: [
                { namePrefix: "ICar" },
                { namePrefix: "icarmeter" }
            ],
            optionalServices: [BLE_SERVICE_UUID]
        });

        state.connectedDevice = device;
        device.addEventListener('gattserverdisconnected', onBleDisconnected);

        updateBleStatusUI("connecting", "Đang kết nối...");
        updateSyncBarUI(false, "Đang kết nối Bluetooth Low Energy...");
        const server = await device.gatt.connect();
        state.gattServer = server;

        // Retrieve GATT Service & Characteristics
        try {
            const service = await server.getPrimaryService(BLE_SERVICE_UUID);
            state.charTpmsConfig  = await service.getCharacteristic(CHAR_TPMS_CONFIG_UUID).catch(() => null);
            state.charGpsFeed     = await service.getCharacteristic(CHAR_GPS_FEED_UUID).catch(() => null);
            state.charSystemCtrl  = await service.getCharacteristic(CHAR_SYSTEM_CTRL_UUID).catch(() => null);
            state.charTripStats   = await service.getCharacteristic(CHAR_TRIP_STATS_UUID).catch(() => null);
        } catch (e) {
            console.warn("Custom service characteristics query:", e);
        }

        updateBleStatusUI("connected", device.name || "ICarMeter Pi");
        showToast(`Đã kết nối: ${device.name}`);

        // AUTOMATIC SYNC: Read entire configuration directly from Pi!
        await fetchConfigFromPi();
    } catch (err) {
        console.error("BLE connection error:", err);
        updateBleStatusUI("disconnected", "Chưa kết nối");
        updateSyncBarUI(false, "Chưa kết nối Pi");
        if (err.name !== 'NotFoundError') {
            showToast(`Lỗi kết nối: ${err.message}`);
        }
    }
}

function disconnectBle() {
    if (state.connectedDevice && state.connectedDevice.gatt.connected) {
        state.connectedDevice.gatt.disconnect();
    }
    onBleDisconnected();
}

function onBleDisconnected() {
    state.gattServer = null;
    state.charTpmsConfig = null;
    state.charGpsFeed = null;
    state.charSystemCtrl = null;
    state.charTripStats = null;
    state.isSyncedWithPi = false;

    updateBleStatusUI("disconnected", "Chưa kết nối");
    updateSyncBarUI(false, "Nguồn dữ liệu: Đã ngắt kết nối với Pi");
    const meta = document.getElementById('tpmsSyncTimestamp');
    if (meta) meta.textContent = "Chưa kết nối Pi";

    showToast("Đã ngắt kết nối với ICarMeter");
}

function updateBleStatusUI(status, label) {
    const badge = document.getElementById('bleStatusBadge');
    const text  = document.getElementById('bleStatusText');
    const devLabel = document.getElementById('connectedDeviceName');
    const btnText = document.getElementById('btnConnectText');

    if (status === 'connected') {
        badge.classList.add('connected');
        text.textContent = "Đã kết nối";
        devLabel.textContent = label;
        btnText.textContent = "Ngắt Kết Nối ICarMeter";
    } else if (status === 'connecting') {
        badge.classList.remove('connected');
        text.textContent = label;
        btnText.textContent = "Đang kết nối...";
    } else {
        badge.classList.remove('connected');
        text.textContent = "Chưa kết nối";
        devLabel.textContent = "Pi Chưa Ghép Nối";
        btnText.textContent = "Quét & Kết nối ICarMeter";
    }
}

function updateSyncBarUI(synced, message) {
    const bar = document.getElementById('syncBar');
    const text = document.getElementById('syncStatusText');
    if (!bar || !text) return;

    if (synced) {
        bar.classList.add('synced');
    } else {
        bar.classList.remove('synced');
    }
    text.textContent = message;
}

// ==============================================================================
// Pi-Centric Sync: Read Config from Pi
// ==============================================================================
async function fetchConfigFromPi() {
    if (!state.charTpmsConfig || !state.gattServer || !state.gattServer.connected) {
        // Fallback for offline demonstration
        updateSyncBarUI(false, "Chưa kết nối Pi. Hãy bật Bluetooth để kéo cấu hình từ xe.");
        showToast("Vui lòng kết nối Bluetooth với ICarMeter trước!");
        return;
    }

    try {
        updateSyncBarUI(false, "Đang tải cấu hình gốc từ Pi...");
        const value = await state.charTpmsConfig.readValue();
        const decoder = new TextDecoder('utf-8');
        const jsonStr = decoder.decode(value);
        const data = JSON.parse(jsonStr);

        // Apply Pi config directly to App UI
        if (data.tpms) {
            state.piConfig.tpms.fl = data.tpms.fl || "";
            state.piConfig.tpms.fr = data.tpms.fr || "";
            state.piConfig.tpms.rl = data.tpms.rl || "";
            state.piConfig.tpms.rr = data.tpms.rr || "";
            state.piConfig.tpms.alert = data.tpms.low_pressure_alert || data.tpms.alert || 2.2;

            document.getElementById('inputFlSensor').value = state.piConfig.tpms.fl;
            document.getElementById('inputFrSensor').value = state.piConfig.tpms.fr;
            document.getElementById('inputRlSensor').value = state.piConfig.tpms.rl;
            document.getElementById('inputRrSensor').value = state.piConfig.tpms.rr;
            document.getElementById('inputPressureAlert').value = state.piConfig.tpms.alert;
        }

        if (data.gps) {
            state.piConfig.gps.enabled = !!data.gps.enabled;
            const toggle = document.getElementById('toggleHardwareGps');
            if (toggle) toggle.checked = state.piConfig.gps.enabled;
            updateGpsModeUI();
        }

        state.isSyncedWithPi = true;
        const nowStr = new Date().toLocaleTimeString('vi-VN', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
        updateSyncBarUI(true, `Nguồn chuẩn: Đã đồng bộ từ Pi (${nowStr})`);
        
        const meta = document.getElementById('tpmsSyncTimestamp');
        if (meta) meta.textContent = `Đã khớp Pi (${nowStr})`;

        showToast("Đã tải cấu hình gốc từ Pi thành công!");
    } catch (err) {
        console.error("Fetch from Pi failed:", err);
        updateSyncBarUI(false, "Lỗi khi đọc cấu hình từ Pi");
        showToast(`Không thể đọc cấu hình từ Pi: ${err.message}`);
    }
}

// ==============================================================================
// Pi-Centric Sync: Save & Push Config to Pi (1-Tap Synchronize)
// ==============================================================================
async function saveAndSyncTpmsToPi() {
    if (!state.charTpmsConfig || !state.gattServer || !state.gattServer.connected) {
        showToast("Chưa kết nối Pi! Vui lòng kết nối Bluetooth để lưu cấu hình trực tiếp vào Pi.");
        return;
    }

    // Read inputs from screen
    const fl = document.getElementById('inputFlSensor').value.trim().toUpperCase();
    const fr = document.getElementById('inputFrSensor').value.trim().toUpperCase();
    const rl = document.getElementById('inputRlSensor').value.trim().toUpperCase();
    const rr = document.getElementById('inputRrSensor').value.trim().toUpperCase();
    const alertPressure = parseFloat(document.getElementById('inputPressureAlert').value) || 2.2;
    const gpsHwEnabled = document.getElementById('toggleHardwareGps')?.checked || false;

    const payload = JSON.stringify({
        action: "update_config",
        fl: fl,
        fr: fr,
        rl: rl,
        rr: rr,
        alert: alertPressure,
        gps_enabled: gpsHwEnabled
    });

    try {
        const encoder = new TextEncoder();
        await state.charTpmsConfig.writeValue(encoder.encode(payload));

        state.piConfig.tpms.fl = fl;
        state.piConfig.tpms.fr = fr;
        state.piConfig.tpms.rl = rl;
        state.piConfig.tpms.rr = rr;
        state.piConfig.tpms.alert = alertPressure;
        state.piConfig.gps.enabled = gpsHwEnabled;
        state.isSyncedWithPi = true;

        const nowStr = new Date().toLocaleTimeString('vi-VN', { hour: '2-digit', minute: '2-digit' });
        updateSyncBarUI(true, `Nguồn chuẩn: Đã lưu vào Pi (${nowStr})`);
        
        const meta = document.getElementById('tpmsSyncTimestamp');
        if (meta) meta.textContent = `Đã lưu trên Pi (${nowStr})`;

        showToast("Cấu hình đã được lưu vĩnh viễn trên Pi thành công!");
    } catch (err) {
        console.error("Save to Pi failed:", err);
        showToast(`Lỗi khi ghi cấu hình vào Pi: ${err.message}`);
    }
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

    // Forward to Pi over BLE (only if hardware GPS on Pi is disabled)
    if (!state.piConfig.gps.enabled) {
        maybeTransmitGpsToPi(coords, speedKmh);
    }
}

function onGpsError(err) {
    console.warn("GPS error:", err.message);
}

function maybeTransmitGpsToPi(coords, speedKmh) {
    const now = Date.now();
    const intervalMs = 1000 / state.sendRateHz;

    if (now - state.lastSendTime < intervalMs) {
        return; // Throttle to target Hz to preserve phone battery
    }
    state.lastSendTime = now;
    state.sendCount++;
    document.getElementById('telSendRate').textContent = `${state.sendRateHz} Hz`;

    if (!state.charGpsFeed || !state.gattServer || !state.gattServer.connected) {
        return;
    }

    // Compact telemetry JSON payload
    const payload = JSON.stringify({
        spd: speedKmh,
        lat: parseFloat(coords.latitude.toFixed(6)),
        lon: parseFloat(coords.longitude.toFixed(6)),
        alt: coords.altitude ? Math.round(coords.altitude) : 0,
        hdg: coords.heading ? Math.round(coords.heading) : 0
    });

    const encoder = new TextEncoder();
    state.charGpsFeed.writeValueWithoutResponse(encoder.encode(payload))
        .catch(err => console.debug("BLE GPS write err:", err));
}

// GPS Option Toggle: Hardware GPS vs Phone BLE GPS
function onGpsModeToggle(checkbox) {
    state.piConfig.gps.enabled = checkbox.checked;
    updateGpsModeUI();
    saveAndSyncTpmsToPi();
}

function updateGpsModeUI() {
    const badge = document.getElementById('gpsSourceBadge');
    const title = document.getElementById('gpsToggleTitle');
    const desc  = document.getElementById('gpsToggleDesc');

    if (state.piConfig.gps.enabled) {
        badge.textContent = "MODULE PI (HW)";
        badge.classList.add('hardware-gps');
        title.textContent = "Dùng Module GPS Trên Pi";
        desc.textContent = "Đang BẬT: ICarMeter sẽ tự đọc cổng serial phần cứng trên Pi. Điện thoại tạm dừng truyền GPS để tiết kiệm pin.";
    } else {
        badge.textContent = "ĐIỆN THOẠI (BLE)";
        badge.classList.remove('hardware-gps');
        title.textContent = "Dùng GPS Từ Điện Thoại (BLE)";
        desc.textContent = "Đang TẮT: Điện thoại lấy GPS độ chính xác cao và truyền liên tục sang ICarMeter qua Bluetooth Low Energy.";
    }
}

// ==============================================================================
// Trip History & Logging (Recorded & Synced with Pi)
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
