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
            mediaPlaybackRequiresUserGesture = false
            cacheMode = WebSettings.LOAD_DEFAULT
        }

        // Bridge native BLE TPMS advertising to Web companion UI
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
