package com.icarmeter.companion

import android.Manifest
import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothManager
import android.bluetooth.le.AdvertiseCallback
import android.bluetooth.le.AdvertiseData
import android.bluetooth.le.AdvertiseSettings
import android.bluetooth.le.BluetoothLeAdvertiser
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.location.Location
import android.location.LocationListener
import android.location.LocationManager
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.os.Build
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import androidx.core.content.ContextCompat
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.EventChannel
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel

class MainActivity : FlutterActivity() {

    private val METHOD_CHANNEL = "com.icarmeter.companion/bridge"
    private val BT_EVENT_CHANNEL = "com.icarmeter.companion/bt_events"
    private val LOCATION_EVENT_CHANNEL = "com.icarmeter.companion/location_events"
    private val WIFI_EVENT_CHANNEL = "com.icarmeter.companion/wifi_events"

    private var btEventSink: EventChannel.EventSink? = null
    private var locationEventSink: EventChannel.EventSink? = null
    private var wifiEventSink: EventChannel.EventSink? = null

    private val mainHandler = Handler(Looper.getMainLooper())

    private var btSppManager: BtSppManager? = null
    private var advertiser: BluetoothLeAdvertiser? = null
    private var advertiseCallback: AdvertiseCallback? = null

    @Volatile
    private var activeWifiNetwork: Network? = null
    private var networkCallback: ConnectivityManager.NetworkCallback? = null

    private var locationManager: LocationManager? = null
    private var locationListener: LocationListener? = null

    private val bluetoothAdapter: BluetoothAdapter?
        get() {
            val manager = getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager
            return manager?.adapter ?: BluetoothAdapter.getDefaultAdapter()
        }

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)

        btSppManager = BtSppManager(context) { type, params ->
            val event = mutableMapOf<String, Any?>("type" to type)
            event.putAll(params)
            emitBtEvent(event)
        }

        // Event Channels
        EventChannel(flutterEngine.dartExecutor.binaryMessenger, BT_EVENT_CHANNEL)
            .setStreamHandler(object : EventChannel.StreamHandler {
                override fun onListen(arguments: Any?, events: EventChannel.EventSink?) {
                    btEventSink = events
                }

                override fun onCancel(arguments: Any?) {
                    btEventSink = null
                }
            })

        EventChannel(flutterEngine.dartExecutor.binaryMessenger, LOCATION_EVENT_CHANNEL)
            .setStreamHandler(object : EventChannel.StreamHandler {
                override fun onListen(arguments: Any?, events: EventChannel.EventSink?) {
                    locationEventSink = events
                }

                override fun onCancel(arguments: Any?) {
                    locationEventSink = null
                }
            })

        EventChannel(flutterEngine.dartExecutor.binaryMessenger, WIFI_EVENT_CHANNEL)
            .setStreamHandler(object : EventChannel.StreamHandler {
                override fun onListen(arguments: Any?, events: EventChannel.EventSink?) {
                    wifiEventSink = events
                }

                override fun onCancel(arguments: Any?) {
                    wifiEventSink = null
                }
            })

        // Method Channel
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, METHOD_CHANNEL)
            .setMethodCallHandler { call: MethodCall, result: MethodChannel.Result ->
                handleMethodCall(call, result)
            }
    }

    private fun emitBtEvent(data: Map<String, Any?>) {
        mainHandler.post {
            btEventSink?.success(data)
        }
    }

    private fun handleMethodCall(call: MethodCall, result: MethodChannel.Result) {
        when (call.method) {
            "connectBtSpp" -> {
                val mac = call.argument<String>("mac")
                val uuid = call.argument<String>("uuid")
                if (mac.isNullOrEmpty()) {
                    result.error("INVALID_ARG", "MAC is required", null)
                    return
                }
                btSppManager?.connect(mac, uuid) { success, err ->
                    mainHandler.post {
                        if (success) {
                            result.success(true)
                        } else {
                            result.error("BT_CONNECT_ERROR", err ?: "Connection failed", null)
                        }
                    }
                }
            }

            "disconnectBtSpp" -> {
                btSppManager?.disconnect()
                result.success(true)
            }

            "btSend" -> {
                val msg = call.argument<String>("message") ?: ""
                val ok = btSppManager?.send(msg) ?: false
                result.success(ok)
            }

            "isBtSppConnected" -> {
                result.success(btSppManager?.isConnected ?: false)
            }

            "getDeviceBluetoothAddress" -> {
                result.success(getBtAddress())
            }

            "isBleAdvertisingSupported" -> {
                val adapter = bluetoothAdapter
                val supported = adapter != null && adapter.isEnabled && adapter.bluetoothLeAdvertiser != null
                result.success(supported)
            }

            "startBleAdvertising" -> {
                val phoneMac = call.argument<String>("phoneMac") ?: ""
                val pressureBar = call.argument<Double>("pressureBar") ?: 2.3
                val tempC = call.argument<Double>("tempC") ?: 25.0
                startBleAdvertisingInternal(phoneMac, pressureBar, tempC, result)
            }

            "stopBleAdvertising" -> {
                stopBleAdvertisingInternal()
                result.success(true)
            }

            "bindToCarWifi" -> {
                bindToCarWifiInternal(result)
            }

            "startGps" -> {
                val rateHz = call.argument<Int>("rateHz") ?: 2
                startGpsInternal(rateHz, result)
            }

            "stopGps" -> {
                stopGpsInternal()
                result.success(true)
            }

            "startForegroundService" -> {
                try {
                    val hasLocation = ContextCompat.checkSelfPermission(
                        this,
                        Manifest.permission.ACCESS_FINE_LOCATION
                    ) == PackageManager.PERMISSION_GRANTED || ContextCompat.checkSelfPermission(
                        this,
                        Manifest.permission.ACCESS_COARSE_LOCATION
                    ) == PackageManager.PERMISSION_GRANTED

                    if (!hasLocation) {
                        result.success(false)
                        return
                    }

                    val intent = Intent(this, GpsBleService::class.java)
                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                        startForegroundService(intent)
                    } else {
                        startService(intent)
                    }
                    result.success(true)
                } catch (e: Exception) {
                    result.error("SERVICE_ERROR", e.message, null)
                }
            }

            "stopForegroundService" -> {
                try {
                    val intent = Intent(this, GpsBleService::class.java)
                    stopService(intent)
                    result.success(true)
                } catch (_: Exception) {
                    result.success(false)
                }
            }

            else -> result.notImplemented()
        }
    }

    private fun getBtAddress(): String {
        try {
            val addr = android.provider.Settings.Secure.getString(contentResolver, "bluetooth_address")
            if (!addr.isNullOrEmpty() && addr != "02:00:00:00:00:00") {
                return addr.uppercase()
            }
        } catch (_: Exception) {}
        return ""
    }

    private fun startBleAdvertisingInternal(
        phoneMac: String,
        pressureBar: Double,
        tempC: Double,
        result: MethodChannel.Result
    ) {
        try {
            val adapter = bluetoothAdapter
            val adv = adapter?.bluetoothLeAdvertiser
            if (adv == null) {
                result.error("UNSUPPORTED", "Device does not support BLE advertising", null)
                return
            }
            advertiser = adv
            stopBleAdvertisingInternal()

            val settings = AdvertiseSettings.Builder()
                .setAdvertiseMode(AdvertiseSettings.ADVERTISE_MODE_LOW_LATENCY)
                .setTxPowerLevel(AdvertiseSettings.ADVERTISE_TX_POWER_HIGH)
                .setConnectable(false)
                .setTimeout(0)
                .build()

            val pRaw = (pressureBar * 100_000.0).toInt()
            val tRaw = (tempC * 100.0).toInt()
            val macBytes = parseMacBytes(phoneMac)

            val data = ByteArray(16)
            data[0] = 0x00
            data[1] = 0x00
            data[2] = (pRaw and 0xFF).toByte()
            data[3] = ((pRaw shr 8) and 0xFF).toByte()
            data[4] = ((pRaw shr 16) and 0xFF).toByte()
            data[5] = ((pRaw shr 24) and 0xFF).toByte()
            data[6] = (tRaw and 0xFF).toByte()
            data[7] = ((tRaw shr 8) and 0xFF).toByte()
            data[8] = ((tRaw shr 16) and 0xFF).toByte()
            data[9] = ((tRaw shr 24) and 0xFF).toByte()
            System.arraycopy(macBytes, 0, data, 10, 6)

            val advertiseData = AdvertiseData.Builder()
                .setIncludeDeviceName(false)
                .setIncludeTxPowerLevel(false)
                .addManufacturerData(0xACAC, data)
                .build()

            val cb = object : AdvertiseCallback() {
                override fun onStartSuccess(settingsInEffect: AdvertiseSettings?) {
                    super.onStartSuccess(settingsInEffect)
                }

                override fun onStartFailure(errorCode: Int) {
                    super.onStartFailure(errorCode)
                }
            }
            advertiseCallback = cb
            adv.startAdvertising(settings, advertiseData, cb)
            result.success(true)
        } catch (e: Exception) {
            result.error("BLE_ADV_ERROR", e.message, null)
        }
    }

    private fun stopBleAdvertisingInternal() {
        try {
            advertiseCallback?.let { advertiser?.stopAdvertising(it) }
            advertiseCallback = null
        } catch (_: Exception) {}
    }

    private fun parseMacBytes(macStr: String): ByteArray {
        val clean = macStr.replace(":", "").replace("-", "").replace(" ", "").trim().uppercase()
        val bytes = ByteArray(6)
        if (clean.length >= 12) {
            for (i in 0 until 6) {
                val sub = clean.substring(i * 2, i * 2 + 2)
                try {
                    bytes[i] = sub.toInt(16).toByte()
                } catch (_: Exception) {
                    bytes[i] = 0
                }
            }
        }
        return bytes
    }

    private fun bindToCarWifiInternal(result: MethodChannel.Result) {
        try {
            val cm = getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager
            if (cm == null) {
                result.success(false)
                return
            }
            if (networkCallback != null) {
                result.success(true)
                return
            }

            val request = NetworkRequest.Builder()
                .addTransportType(NetworkCapabilities.TRANSPORT_WIFI)
                .removeCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
                .build()

            val cb = object : ConnectivityManager.NetworkCallback() {
                override fun onAvailable(network: Network) {
                    super.onAvailable(network)
                    activeWifiNetwork = network
                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                        cm.bindProcessToNetwork(network)
                    }
                    mainHandler.post {
                        wifiEventSink?.success(mapOf("connected" to true))
                    }
                }

                override fun onLost(network: Network) {
                    super.onLost(network)
                    if (activeWifiNetwork == network) {
                        activeWifiNetwork = null
                        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                            cm.bindProcessToNetwork(null)
                        }
                    }
                    mainHandler.post {
                        wifiEventSink?.success(mapOf("connected" to false))
                    }
                }
            }
            networkCallback = cb
            cm.requestNetwork(request, cb)
            cm.registerNetworkCallback(request, cb)
            result.success(true)
        } catch (e: Exception) {
            result.error("WIFI_ERROR", e.message, null)
        }
    }

    private fun startGpsInternal(rateHz: Int, result: MethodChannel.Result) {
        try {
            if (ContextCompat.checkSelfPermission(this, Manifest.permission.ACCESS_FINE_LOCATION)
                != PackageManager.PERMISSION_GRANTED
            ) {
                result.error("PERMISSION_DENIED", "Location permission not granted", null)
                return
            }

            locationManager = getSystemService(Context.LOCATION_SERVICE) as? LocationManager
            val minTimeMs = (1000L / maxOf(1, rateHz)).coerceAtLeast(100L)

            val listener = object : LocationListener {
                override fun onLocationChanged(loc: Location) {
                    mainHandler.post {
                        locationEventSink?.success(
                            mapOf(
                                "speed" to loc.speed.toDouble(),
                                "latitude" to loc.latitude,
                                "longitude" to loc.longitude,
                                "altitude" to loc.altitude,
                                "heading" to loc.bearing.toDouble(),
                                "accuracy" to loc.accuracy.toDouble()
                            )
                        )
                    }
                }

                override fun onStatusChanged(provider: String?, status: Int, extras: Bundle?) {}
                override fun onProviderEnabled(provider: String) {}
                override fun onProviderDisabled(provider: String) {}
            }

            locationListener = listener
            if (locationManager?.isProviderEnabled(LocationManager.GPS_PROVIDER) == true) {
                locationManager?.requestLocationUpdates(
                    LocationManager.GPS_PROVIDER,
                    minTimeMs,
                    0f,
                    listener,
                    Looper.getMainLooper()
                )
            }
            result.success(true)
        } catch (e: Exception) {
            result.error("GPS_ERROR", e.message, null)
        }
    }

    private fun stopGpsInternal() {
        try {
            locationListener?.let { locationManager?.removeUpdates(it) }
            locationListener = null
        } catch (_: Exception) {}
    }

    override fun onDestroy() {
        try {
            unregisterReceiver(btReceiver)
        } catch (_: Exception) {}
        btSppManager?.disconnect()
        stopBleAdvertisingInternal()
        stopGpsInternal()
        super.onDestroy()
    }
}
