/**
 * ICarMeter Companion App - Pure Flat Architecture
 * Handles BLE Communication, Phone GPS Forwarding, TPMS Setup & Trip History.
 */

// ICarMeter Custom BLE GATT UUIDs
const BLE_SERVICE_UUID     = "1c480001-0000-1000-8000-00805f9b34fb";
const CHAR_TPMS_CONFIG_UUID = "1c480002-0000-1000-8000-00805f9b34fb";
const CHAR_GPS_FEED_UUID   = "1c480003-0000-1000-8000-00805f9b34fb";
const CHAR_SYSTEM_CTRL_UUID= "1c480004-0000-1000-8000-00805f9b34fb";

// App State
const state = {
    connectedDevice: null,
    gattServer: null,
    charTpmsConfig: null,
    charGpsFeed: null,
    charSystemCtrl: null,
    
    // GPS & Telemetry
    hardwareGpsEnabled: false, // false = Use Phone GPS (BLE), true = Use Pi Hardware GPS
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
    
    // Storage
    tpmsConfig: {
        fl: "A4:C1:38:00:00:01",
        fr: "A4:C1:38:00:00:02",
        rl: "A4:C1:38:00:00:03",
        rr: "A4:C1:38:00:00:04",
        alertPressure: 2.2
    },
    trips: []
};

// ==============================================================================
// Initialization & Local Storage
// ==============================================================================
document.addEventListener('DOMContentLoaded', () => {
    loadLocalSettings();
    renderTpmsInputs();
    renderTripHistory();
    startGpsTracker();
});

function loadLocalSettings() {
    try {
        const savedTpms = localStorage.getItem('icarmeter_tpms');
        if (savedTpms) {
            state.tpmsConfig = JSON.parse(savedTpms);
        }

        const savedGpsMode = localStorage.getItem('icarmeter_hw_gps');
        if (savedGpsMode !== null) {
            state.hardwareGpsEnabled = savedGpsMode === 'true';
            const toggle = document.getElementById('toggleHardwareGps');
            if (toggle) toggle.checked = state.hardwareGpsEnabled;
            updateGpsModeUI();
        }

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
    showToast("Đã lưu cài đặt");
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
// Bluetooth Low Energy (BLE) Client
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
        updateBleStatusUI("connecting", "Đang quét...");

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
        const server = await device.gatt.connect();
        state.gattServer = server;

        // Retrieve GATT Service & Characteristics
        try {
            const service = await server.getPrimaryService(BLE_SERVICE_UUID);
            state.charTpmsConfig = await service.getCharacteristic(CHAR_TPMS_CONFIG_UUID).catch(() => null);
            state.charGpsFeed    = await service.getCharacteristic(CHAR_GPS_FEED_UUID).catch(() => null);
            state.charSystemCtrl = await service.getCharacteristic(CHAR_SYSTEM_CTRL_UUID).catch(() => null);
        } catch (e) {
            console.warn("Custom service not found on device, connected in generic mode:", e);
        }

        updateBleStatusUI("connected", device.name || "ICarMeter Pi");
        showToast(`Đã kết nối thành công: ${device.name}`);

        // Sync initial GPS mode to Pi
        sendGpsModeToPi();
    } catch (err) {
        console.error("BLE connection failed:", err);
        updateBleStatusUI("disconnected", "Chưa kết nối");
        if (err.name !== 'NotFoundError') { // Not canceled by user
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
    updateBleStatusUI("disconnected", "Chưa kết nối");
    showToast("Đã ngắt kết nối với xe");
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

// ==============================================================================
// GPS Tracking & BLE Forwarding (Tethering)
// ==============================================================================
function startGpsTracker() {
    if (!('geolocation' in navigator)) {
        showToast("Điện thoại không có dịch vụ định vị GPS.");
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

    // If trip recording is active, sample speed
    if (state.isRecordingTrip) {
        sampleTripTelemetry(speedKmh);
    }

    // Forward to Pi over BLE (if enabled and connected)
    if (!state.hardwareGpsEnabled) {
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
        return; // Throttle to target Hz to preserve battery
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
    state.hardwareGpsEnabled = checkbox.checked;
    localStorage.setItem('icarmeter_hw_gps', state.hardwareGpsEnabled.toString());
    updateGpsModeUI();
    sendGpsModeToPi();
}

function updateGpsModeUI() {
    const badge = document.getElementById('gpsSourceBadge');
    const title = document.getElementById('gpsToggleTitle');
    const desc  = document.getElementById('gpsToggleDesc');

    if (state.hardwareGpsEnabled) {
        badge.textContent = "MODULE PI (HW)";
        badge.classList.add('hardware-gps');
        title.textContent = "Dùng Module GPS";
        desc.textContent = "Đang BẬT: ICarMeter sẽ tự đọc cổng serial phần cứng.";
    } else {
        badge.textContent = "ĐIỆN THOẠI (BLE)";
        badge.classList.remove('hardware-gps');
        title.textContent = "Dùng GPS Từ Điện Thoại";
        desc.textContent = "Đang TẮT: Đọc GPS từ điện thoại.";
    }
}

function sendGpsModeToPi() {
    if (!state.charSystemCtrl || !state.gattServer || !state.gattServer.connected) {
        return;
    }

    const payload = JSON.stringify({
        gps_mode: state.hardwareGpsEnabled ? "hardware" : "phone"
    });

    const encoder = new TextEncoder();
    state.charSystemCtrl.writeValue(encoder.encode(payload))
        .then(() => showToast(`Đã đồng bộ chế độ GPS: ${state.hardwareGpsEnabled ? "Module Pi" : "Điện thoại BLE"}`))
        .catch(err => console.warn("Failed to sync GPS mode to Pi:", err));
}

// ==============================================================================
// TPMS Configuration
// ==============================================================================
function renderTpmsInputs() {
    document.getElementById('inputFlSensor').value = state.tpmsConfig.fl || '';
    document.getElementById('inputFrSensor').value = state.tpmsConfig.fr || '';
    document.getElementById('inputRlSensor').value = state.tpmsConfig.rl || '';
    document.getElementById('inputRrSensor').value = state.tpmsConfig.rr || '';
    document.getElementById('inputPressureAlert').value = state.tpmsConfig.alertPressure || 2.2;
}

function readTpmsInputs() {
    state.tpmsConfig.fl = document.getElementById('inputFlSensor').value.trim().toUpperCase();
    state.tpmsConfig.fr = document.getElementById('inputFrSensor').value.trim().toUpperCase();
    state.tpmsConfig.rl = document.getElementById('inputRlSensor').value.trim().toUpperCase();
    state.tpmsConfig.rr = document.getElementById('inputRrSensor').value.trim().toUpperCase();
    state.tpmsConfig.alertPressure = parseFloat(document.getElementById('inputPressureAlert').value) || 2.2;
}

function saveTpmsLocal() {
    readTpmsInputs();
    localStorage.setItem('icarmeter_tpms', JSON.stringify(state.tpmsConfig));
    showToast("Đã lưu cấu hình 4 van xe trên điện thoại");
}

async function syncTpmsToPi() {
    saveTpmsLocal();

    if (!state.charTpmsConfig || !state.gattServer || !state.gattServer.connected) {
        showToast("Chưa kết nối Bluetooth với ICarMeter! Hãy kết nối ở tab Đồng Hồ trước.");
        return;
    }

    try {
        const payload = JSON.stringify({
            fl: state.tpmsConfig.fl,
            fr: state.tpmsConfig.fr,
            rl: state.tpmsConfig.rl,
            rr: state.tpmsConfig.rr,
            alert: state.tpmsConfig.alertPressure
        });

        const encoder = new TextEncoder();
        await state.charTpmsConfig.writeValue(encoder.encode(payload));
        showToast("Đã gửi thành công cấu hình TPMS xuống ICarMeter!");
    } catch (err) {
        console.error("TPMS BLE Sync error:", err);
        showToast(`Lỗi truyền Bluetooth: ${err.message}`);
    }
}

// ==============================================================================
// Trip History & Logging
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
    showToast("Đã bắt đầu ghi nhận hành trình mới");
}

function sampleTripTelemetry(speedKmh) {
    state.tripSpeedSamples.push(speedKmh);
    if (speedKmh > state.tripMaxSpeed) {
        state.tripMaxSpeed = speedKmh;
    }

    // Approximate distance delta (km) = (v / 3600) * dt
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

    // Save trip object
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
