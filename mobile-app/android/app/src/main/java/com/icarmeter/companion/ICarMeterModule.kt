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
import android.os.Looper
import androidx.core.content.ContextCompat
import com.facebook.react.bridge.*
import com.facebook.react.modules.core.DeviceEventManagerModule
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL

class ICarMeterModule(private val reactContext: ReactApplicationContext) :
    ReactContextBaseJavaModule(reactContext) {

    private var advertiser: BluetoothLeAdvertiser? = null
    private var advertiseCallback: AdvertiseCallback? = null

    @Volatile
    private var activeWifiNetwork: Network? = null
    private var networkCallback: ConnectivityManager.NetworkCallback? = null

    private var locationManager: LocationManager? = null
    private var locationListener: LocationListener? = null

    override fun getName(): String = "ICarMeterBridge"

    private fun sendEvent(eventName: String, params: WritableMap?) {
        if (reactContext.hasActiveReactInstance()) {
            reactContext
                .getJSModule(DeviceEventManagerModule.RCTDeviceEventEmitter::class.java)
                .emit(eventName, params)
        }
    }

    @ReactMethod
    fun addListener(eventName: String) {
        // Required for React Native NativeEventEmitter
    }

    @ReactMethod
    fun removeListeners(count: Int) {
        // Required for React Native NativeEventEmitter
    }

    @ReactMethod
    fun isBleAdvertisingSupported(promise: Promise) {
        try {
            val bluetoothManager =
                reactContext.getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager
            val adapter = bluetoothManager?.adapter
            val supported = adapter != null && adapter.isEnabled && adapter.bluetoothLeAdvertiser != null
            promise.resolve(supported)
        } catch (e: Exception) {
            promise.resolve(false)
        }
    }

    @ReactMethod
    fun getDeviceBluetoothAddress(promise: Promise) {
        try {
            // 1. Try Settings.Secure "bluetooth_address"
            val btAddress = android.provider.Settings.Secure.getString(
                reactContext.contentResolver,
                "bluetooth_address"
            )
            if (!btAddress.isNullOrEmpty() && btAddress != "02:00:00:00:00:00") {
                promise.resolve(btAddress.uppercase())
                return
            }

            // 2. Try reflection on BluetoothAdapter internal service
            try {
                val adapter = BluetoothAdapter.getDefaultAdapter()
                if (adapter != null) {
                    val mServiceField = adapter.javaClass.getDeclaredField("mService")
                    mServiceField.isAccessible = true
                    val mService = mServiceField.get(adapter)
                    if (mService != null) {
                        val getAddressMethod = mService.javaClass.getMethod("getAddress")
                        val addr = getAddressMethod.invoke(mService) as? String
                        if (!addr.isNullOrEmpty() && addr != "02:00:00:00:00:00") {
                            promise.resolve(addr.uppercase())
                            return
                        }
                    }
                }
            } catch (_: Exception) {}

            // 3. Try standard adapter.address
            try {
                @Suppress("DEPRECATION")
                val adapter = BluetoothAdapter.getDefaultAdapter()
                val addr = adapter?.address
                if (!addr.isNullOrEmpty() && addr != "02:00:00:00:00:00") {
                    promise.resolve(addr.uppercase())
                    return
                }
            } catch (_: Exception) {}

            // No fallback - return empty string if not obtainable
            promise.resolve("")
        } catch (e: Exception) {
            promise.resolve("")
        }
    }

    @ReactMethod
    fun startBleAdvertising(phoneMac: String, pressureBar: Double, tempC: Double, promise: Promise) {
        try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                if (ContextCompat.checkSelfPermission(
                        reactContext,
                        Manifest.permission.BLUETOOTH_ADVERTISE
                    ) != PackageManager.PERMISSION_GRANTED
                ) {
                    promise.reject("PERMISSION_DENIED", "BLUETOOTH_ADVERTISE permission required")
                    return
                }
            }

            val bluetoothManager =
                reactContext.getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager
            val adapter = bluetoothManager?.adapter
            if (adapter == null || !adapter.isEnabled) {
                promise.reject("BLUETOOTH_OFF", "Bluetooth is turned off")
                return
            }

            val adv = adapter.bluetoothLeAdvertiser
            if (adv == null) {
                promise.reject("UNSUPPORTED", "Device does not support BLE advertising")
                return
            }
            advertiser = adv

            // Stop any existing session
            stopBleAdvertisingInternal()

            val settings = AdvertiseSettings.Builder()
                .setAdvertiseMode(AdvertiseSettings.ADVERTISE_MODE_LOW_LATENCY)
                .setTxPowerLevel(AdvertiseSettings.ADVERTISE_TX_POWER_HIGH)
                .setConnectable(false)
                .setTimeout(0)
                .build()

            // Sysgration 16-byte TPMS payload:
            // 0..1: Status / flags
            // 2..5: 32-bit Pressure in little-endian (Pascals * 100)
            // 6..9: 32-bit Temperature in little-endian (Celsius * 100)
            // 10..15: 6-byte Phone MAC address
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

            val callback = object : AdvertiseCallback() {
                override fun onStartSuccess(settingsInEffect: AdvertiseSettings?) {
                    super.onStartSuccess(settingsInEffect)
                }

                override fun onStartFailure(errorCode: Int) {
                    super.onStartFailure(errorCode)
                }
            }

            advertiseCallback = callback
            adv.startAdvertising(settings, advertiseData, callback)
            promise.resolve(true)
        } catch (e: Exception) {
            promise.reject("BLE_ADV_ERROR", e.message)
        }
    }

    private fun stopBleAdvertisingInternal() {
        try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                if (ContextCompat.checkSelfPermission(
                        reactContext,
                        Manifest.permission.BLUETOOTH_ADVERTISE
                    ) != PackageManager.PERMISSION_GRANTED
                ) {
                    return
                }
            }
            advertiseCallback?.let {
                advertiser?.stopAdvertising(it)
            }
            advertiseCallback = null
        } catch (_: Exception) {}
    }

    @ReactMethod
    fun stopBleAdvertising(promise: Promise) {
        stopBleAdvertisingInternal()
        promise.resolve(true)
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
        } else {
            val raw = clean.toByteArray(Charsets.UTF_8)
            System.arraycopy(raw, 0, bytes, 0, minOf(raw.size, 6))
        }
        return bytes
    }

    @ReactMethod
    fun bindToCarWifi(promise: Promise) {
        try {
            val cm =
                reactContext.getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager
            if (cm == null) {
                promise.resolve(false)
                return
            }

            if (networkCallback != null) {
                promise.resolve(true)
                return
            }

            val request = NetworkRequest.Builder()
                .addTransportType(NetworkCapabilities.TRANSPORT_WIFI)
                .removeCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET)
                .build()

            val callback = object : ConnectivityManager.NetworkCallback() {
                override fun onAvailable(network: Network) {
                    super.onAvailable(network)
                    activeWifiNetwork = network
                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                        cm.bindProcessToNetwork(network)
                    } else {
                        @Suppress("DEPRECATION")
                        ConnectivityManager.setProcessDefaultNetwork(network)
                    }
                    val params = Arguments.createMap().apply {
                        putBoolean("connected", true)
                    }
                    sendEvent("onWifiStateChange", params)
                }

                override fun onLost(network: Network) {
                    super.onLost(network)
                    if (activeWifiNetwork == network) {
                        activeWifiNetwork = null
                        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                            cm.bindProcessToNetwork(null)
                        }
                    }
                    val params = Arguments.createMap().apply {
                        putBoolean("connected", false)
                    }
                    sendEvent("onWifiStateChange", params)
                }
            }

            networkCallback = callback
            cm.requestNetwork(request, callback)
            cm.registerNetworkCallback(request, callback)

            // Probe existing connected Wi-Fi
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                for (net in cm.allNetworks) {
                    val caps = cm.getNetworkCapabilities(net) ?: continue
                    if (caps.hasTransport(NetworkCapabilities.TRANSPORT_WIFI)) {
                        activeWifiNetwork = net
                        cm.bindProcessToNetwork(net)
                        break
                    }
                }
            }

            promise.resolve(true)
        } catch (e: Exception) {
            promise.resolve(false)
        }
    }

    @ReactMethod
    fun startForegroundService(promise: Promise) {
        try {
            val intent = Intent(reactContext, GpsBleService::class.java)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                reactContext.startForegroundService(intent)
            } else {
                reactContext.startService(intent)
            }
            promise.resolve(true)
        } catch (e: Exception) {
            promise.reject("SERVICE_ERROR", e.message)
        }
    }

    @ReactMethod
    fun stopForegroundService(promise: Promise) {
        try {
            val intent = Intent(reactContext, GpsBleService::class.java)
            reactContext.stopService(intent)
            promise.resolve(true)
        } catch (e: Exception) {
            promise.resolve(false)
        }
    }

    @ReactMethod
    fun startGps(rateHz: Int, promise: Promise) {
        try {
            if (ContextCompat.checkSelfPermission(
                    reactContext,
                    Manifest.permission.ACCESS_FINE_LOCATION
                ) != PackageManager.PERMISSION_GRANTED
            ) {
                promise.reject("PERMISSION_DENIED", "Location permission not granted")
                return
            }

            locationManager =
                reactContext.getSystemService(Context.LOCATION_SERVICE) as? LocationManager

            val minTimeMs = (1000L / maxOf(1, rateHz)).coerceAtLeast(100L)

            val listener = object : LocationListener {
                override fun onLocationChanged(loc: Location) {
                    val params = Arguments.createMap().apply {
                        putDouble("speed", loc.speed.toDouble())
                        putDouble("latitude", loc.latitude)
                        putDouble("longitude", loc.longitude)
                        putDouble("altitude", loc.altitude)
                        putDouble("heading", loc.bearing.toDouble())
                        putDouble("accuracy", loc.accuracy.toDouble())
                    }
                    sendEvent("onLocationUpdate", params)
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

            if (locationManager?.isProviderEnabled(LocationManager.NETWORK_PROVIDER) == true) {
                locationManager?.requestLocationUpdates(
                    LocationManager.NETWORK_PROVIDER,
                    minTimeMs,
                    0f,
                    listener,
                    Looper.getMainLooper()
                )
            }

            promise.resolve(true)
        } catch (e: Exception) {
            promise.reject("GPS_ERROR", e.message)
        }
    }

    @ReactMethod
    fun stopGps(promise: Promise) {
        try {
            locationListener?.let {
                locationManager?.removeUpdates(it)
            }
            locationListener = null
            promise.resolve(true)
        } catch (e: Exception) {
            promise.resolve(false)
        }
    }

    @ReactMethod
    fun httpGet(urlStr: String, promise: Promise) {
        Thread {
            try {
                val url = URL(urlStr)
                val conn = if (activeWifiNetwork != null && Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                    activeWifiNetwork!!.openConnection(url) as HttpURLConnection
                } else {
                    url.openConnection() as HttpURLConnection
                }
                conn.connectTimeout = 3000
                conn.readTimeout = 3000
                conn.requestMethod = "GET"
                conn.connect()
                val code = conn.responseCode
                val stream = if (code in 200..299) conn.inputStream else (conn.errorStream ?: conn.inputStream)
                val text = stream.bufferedReader().use { it.readText() }
                conn.disconnect()

                val json = JSONObject().apply {
                    put("status", code)
                    put("body", text)
                }
                promise.resolve(json.toString())
            } catch (e: Exception) {
                val json = JSONObject().apply {
                    put("status", 0)
                    put("error", e.message ?: "Network error")
                }
                promise.resolve(json.toString())
            }
        }.start()
    }

    @ReactMethod
    fun httpPost(urlStr: String, jsonBody: String, promise: Promise) {
        Thread {
            try {
                val url = URL(urlStr)
                val conn = if (activeWifiNetwork != null && Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                    activeWifiNetwork!!.openConnection(url) as HttpURLConnection
                } else {
                    url.openConnection() as HttpURLConnection
                }
                conn.connectTimeout = 3500
                conn.readTimeout = 3500
                conn.requestMethod = "POST"
                conn.setRequestProperty("Content-Type", "application/json; charset=utf-8")
                conn.doOutput = true
                conn.outputStream.use { os ->
                    val bytes = jsonBody.toByteArray(Charsets.UTF_8)
                    os.write(bytes, 0, bytes.size)
                }
                val code = conn.responseCode
                val stream = if (code in 200..299) conn.inputStream else (conn.errorStream ?: conn.inputStream)
                val text = stream.bufferedReader().use { it.readText() }
                conn.disconnect()

                val json = JSONObject().apply {
                    put("status", code)
                    put("body", text)
                }
                promise.resolve(json.toString())
            } catch (e: Exception) {
                val json = JSONObject().apply {
                    put("status", 0)
                    put("error", e.message ?: "Network error")
                }
                promise.resolve(json.toString())
            }
        }.start()
    }
}
