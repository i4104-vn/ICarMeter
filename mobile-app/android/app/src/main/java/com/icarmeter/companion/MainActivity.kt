package com.icarmeter.companion

import android.Manifest
import android.annotation.SuppressLint
import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothManager
import android.bluetooth.le.AdvertiseCallback
import android.bluetooth.le.AdvertiseData
import android.bluetooth.le.AdvertiseSettings
import android.bluetooth.le.BluetoothLeAdvertiser
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.os.Build
import android.os.Bundle
import android.webkit.*
import androidx.appcompat.app.AppCompatActivity
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat

class MainActivity : AppCompatActivity() {

    private lateinit var webView: WebView
    private var bleTpmsInterface: BleTpmsInterface? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Request runtime permissions on modern Android versions
        checkAndRequestPermissions()

        // Bind network sockets to Wi-Fi even if the car hotspot has no upstream internet
        bindToCarWifiNetwork()

        setupWebView()
    }

    @SuppressLint("SetJavaScriptEnabled")
    private fun setupWebView() {
        webView = WebView(this)
        setContentView(webView)

        webView.settings.apply {
            javaScriptEnabled = true
            domStorageEnabled = true
            databaseEnabled = true
            setGeolocationEnabled(true)
            allowFileAccess = true
            allowContentAccess = true
            allowFileAccessFromFileURLs = true
            allowUniversalAccessFromFileURLs = true
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
                mixedContentMode = WebSettings.MIXED_CONTENT_ALWAYS_ALLOW
            }
            mediaPlaybackRequiresUserGesture = false
            cacheMode = WebSettings.LOAD_DEFAULT
        }

        // Bridge native BLE TPMS advertising and native HTTP socket to Web companion UI
        bleTpmsInterface = BleTpmsInterface()
        webView.addJavascriptInterface(bleTpmsInterface!!, "AndroidBleTpms")

        webView.webChromeClient = object : WebChromeClient() {
            override fun onGeolocationPermissionsShowPrompt(
                origin: String?,
                callback: GeolocationPermissions.Callback?
            ) {
                // Grant geolocation access to internal asset app
                callback?.invoke(origin, true, false)
            }

            override fun onPermissionRequest(request: PermissionRequest?) {
                request?.grant(request.resources)
            }
        }

        webView.webViewClient = object : WebViewClient() {
            override fun onPageFinished(view: WebView?, url: String?) {
                super.onPageFinished(view, url)
                // Start background foreground service to ensure persistent GPS transmission
                startGpsForegroundService()
            }
        }

        webView.loadUrl("file:///android_asset/index.html")
    }

    private fun startGpsForegroundService() {
        val intent = Intent(this, GpsBleService::class.java)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            startForegroundService(intent)
        } else {
            startService(intent)
        }
    }

    private fun checkAndRequestPermissions() {
        val permissions = mutableListOf(
            Manifest.permission.ACCESS_FINE_LOCATION,
            Manifest.permission.ACCESS_COARSE_LOCATION
        )

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            permissions.add(Manifest.permission.BLUETOOTH_SCAN)
            permissions.add(Manifest.permission.BLUETOOTH_CONNECT)
            permissions.add(Manifest.permission.BLUETOOTH_ADVERTISE)
        }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            permissions.add(Manifest.permission.POST_NOTIFICATIONS)
        }

        val needed = permissions.filter {
            ContextCompat.checkSelfPermission(this, it) != PackageManager.PERMISSION_GRANTED
        }

        if (needed.isNotEmpty()) {
            ActivityCompat.requestPermissions(this, needed.toTypedArray(), 1001)
        }
    }

    @Volatile
    private var activeWifiNetwork: Network? = null

    private fun findWifiNetwork(): Network? {
        activeWifiNetwork?.let { return it }
        val cm = getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager ?: return null
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            for (net in cm.allNetworks) {
                val caps = cm.getNetworkCapabilities(net) ?: continue
                if (caps.hasTransport(NetworkCapabilities.TRANSPORT_WIFI)) {
                    activeWifiNetwork = net
                    cm.bindProcessToNetwork(net)
                    return net
                }
            }
        }
        return null
    }

    private fun bindToCarWifiNetwork() {
        try {
            val cm = getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager ?: return
            
            // Crucial: remove NET_CAPABILITY_INTERNET so the callback is called for local car AP
            // without upstream internet connection.
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
                    android.util.Log.i("ICarMeter", "Bound app process to Wi-Fi: $network")

                    // Notify companion frontend to reconnect immediately
                    runOnUiThread {
                        webView.evaluateJavascript("if (window.connMgr) window.connMgr.forceReconnect();", null)
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
                }
            }

            cm.requestNetwork(request, callback)
            cm.registerNetworkCallback(request, callback)

            // Probe right now in case Wi-Fi is already connected
            findWifiNetwork()
        } catch (e: Exception) {
            android.util.Log.w("ICarMeter", "bindToCarWifiNetwork error: ${e.message}")
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        bleTpmsInterface?.stopBleAdvertising()
    }

    override fun onBackPressed() {
        if (webView.canGoBack()) {
            webView.goBack()
        } else {
            super.onBackPressed()
        }
    }

    inner class BleTpmsInterface {
        private var advertiser: BluetoothLeAdvertiser? = null
        private var advertiseCallback: AdvertiseCallback? = null

        @JavascriptInterface
        fun isBleAdvertisingSupported(): Boolean {
            val bluetoothManager = getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager
            val adapter = bluetoothManager?.adapter
            return adapter != null && adapter.isEnabled && adapter.bluetoothLeAdvertiser != null
        }

        @JavascriptInterface
        fun startBleAdvertising(phoneMac: String, pressureBar: Float, tempC: Float): Boolean {
            try {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                    if (ContextCompat.checkSelfPermission(this@MainActivity, Manifest.permission.BLUETOOTH_ADVERTISE)
                        != PackageManager.PERMISSION_GRANTED) {
                        return false
                    }
                }

                val bluetoothManager = getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager
                val adapter = bluetoothManager?.adapter ?: return false
                if (!adapter.isEnabled) return false

                val adv = adapter.bluetoothLeAdvertiser ?: return false
                advertiser = adv

                stopBleAdvertising()

                val settings = AdvertiseSettings.Builder()
                    .setAdvertiseMode(AdvertiseSettings.ADVERTISE_MODE_LOW_LATENCY)
                    .setTxPowerLevel(AdvertiseSettings.ADVERTISE_TX_POWER_HIGH)
                    .setConnectable(false)
                    .setTimeout(0)
                    .build()

                // Sysgration 16-byte TPMS payload:
                // Offset 0..1: Status / flags
                // Offset 2..5: 32-bit Pressure in little-endian (Pascals*100)
                // Offset 6..9: 32-bit Temperature in little-endian (Celsius*100)
                // Offset 10..15: 6-byte Phone MAC address
                val pRaw = (pressureBar * 100_000f).toInt()
                val tRaw = (tempC * 100f).toInt()
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
                        android.util.Log.i("BleTpms", "BLE advertising started for MAC $phoneMac ($pressureBar bar, $tempC C)")
                    }

                    override fun onStartFailure(errorCode: Int) {
                        super.onStartFailure(errorCode)
                        android.util.Log.e("BleTpms", "BLE advertising failed: error $errorCode")
                    }
                }

                advertiseCallback = callback
                adv.startAdvertising(settings, advertiseData, callback)
                return true
            } catch (e: Exception) {
                android.util.Log.e("BleTpms", "Exception in startBleAdvertising: ${e.message}")
                return false
            }
        }

        @JavascriptInterface
        fun stopBleAdvertising() {
            try {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                    if (ContextCompat.checkSelfPermission(this@MainActivity, Manifest.permission.BLUETOOTH_ADVERTISE)
                        != PackageManager.PERMISSION_GRANTED) {
                        return
                    }
                }
                advertiseCallback?.let {
                    advertiser?.stopAdvertising(it)
                }
                advertiseCallback = null
            } catch (e: Exception) {
                android.util.Log.e("BleTpms", "Error stopping BLE advertising: ${e.message}")
            }
        }

        @JavascriptInterface
        fun httpGet(urlStr: String): String {
            return try {
                val url = java.net.URL(urlStr)
                val wifiNet = findWifiNetwork()
                val conn = if (wifiNet != null) {
                    wifiNet.openConnection(url) as java.net.HttpURLConnection
                } else {
                    url.openConnection() as java.net.HttpURLConnection
                }
                conn.connectTimeout = 3000
                conn.readTimeout = 3000
                conn.requestMethod = "GET"
                conn.connect()
                val code = conn.responseCode
                val stream = if (code in 200..299) conn.inputStream else (conn.errorStream ?: conn.inputStream)
                val text = stream.bufferedReader().use { it.readText() }
                conn.disconnect()
                org.json.JSONObject().apply {
                    put("status", code)
                    put("body", text)
                }.toString()
            } catch (e: Exception) {
                org.json.JSONObject().apply {
                    put("status", 0)
                    put("error", e.message ?: "Network error")
                }.toString()
            }
        }

        @JavascriptInterface
        fun httpPost(urlStr: String, jsonBody: String): String {
            return try {
                val url = java.net.URL(urlStr)
                val wifiNet = findWifiNetwork()
                val conn = if (wifiNet != null) {
                    wifiNet.openConnection(url) as java.net.HttpURLConnection
                } else {
                    url.openConnection() as java.net.HttpURLConnection
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
                org.json.JSONObject().apply {
                    put("status", code)
                    put("body", text)
                }.toString()
            } catch (e: Exception) {
                org.json.JSONObject().apply {
                    put("status", 0)
                    put("error", e.message ?: "Network error")
                }.toString()
            }
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
    }
}
