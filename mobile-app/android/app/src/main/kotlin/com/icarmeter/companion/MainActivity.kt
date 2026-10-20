package com.icarmeter.companion

import android.Manifest
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
    @Volatile
    private var activeWifiNetwork: Network? = null
    private var networkCallback: ConnectivityManager.NetworkCallback? = null

    private var locationManager: LocationManager? = null
    private var locationListener: LocationListener? = null

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
        btSppManager?.disconnect()
        stopGpsInternal()
        super.onDestroy()
    }
}
