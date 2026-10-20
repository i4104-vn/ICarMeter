package com.icarmeter.companion

import android.annotation.SuppressLint
import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothDevice
import android.bluetooth.BluetoothManager
import android.bluetooth.BluetoothSocket
import android.content.Context
import android.os.Build
import com.facebook.react.bridge.Arguments
import com.facebook.react.bridge.WritableArray
import com.facebook.react.bridge.WritableMap
import java.io.InputStream
import java.io.OutputStream
import java.util.UUID
import java.util.concurrent.Executors

class BtSppManager(
    private val context: Context,
    private val eventEmitter: (String, WritableMap) -> Unit
) {
    companion object {
        val SPP_UUID: UUID = UUID.fromString("00001101-0000-1000-8000-00805F9B34FB")
    }

    private var socket: BluetoothSocket? = null
    private var inputStream: InputStream? = null
    private var outputStream: OutputStream? = null
    private var readerThread: Thread? = null

    @Volatile
    var isConnected: Boolean = false
        private set

    private var connectedMac: String? = null
    private var connectedName: String? = null

    private val executor = Executors.newSingleThreadExecutor()

    private val bluetoothAdapter: BluetoothAdapter?
        get() {
            val manager = context.getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager
            return manager?.adapter ?: BluetoothAdapter.getDefaultAdapter()
        }

    @SuppressLint("MissingPermission")
    fun getPairedDevices(): WritableArray {
        val array = Arguments.createArray()
        val adapter = bluetoothAdapter ?: return array
        if (!adapter.isEnabled) return array

        try {
            val bonded = adapter.bondedDevices ?: return array
            for (device in bonded) {
                val map = Arguments.createMap()
                map.putString("mac", device.address)
                map.putString("name", device.name ?: device.address)
                array.pushMap(map)
            }
        } catch (e: Exception) {
            // Permission or adapter error
        }
        return array
    }

    @SuppressLint("MissingPermission")
    fun connect(mac: String, uuidStr: String?, onResult: (Boolean, String?) -> Unit) {
        executor.execute {
            disconnectInternal(notify = false)

            val adapter = bluetoothAdapter
            if (adapter == null || !adapter.isEnabled) {
                onResult(false, "Bluetooth is turned off")
                return@execute
            }

            try {
                // Always cancel discovery prior to connecting RFCOMM
                adapter.cancelDiscovery()
            } catch (_: Exception) {}

            val device: BluetoothDevice
            try {
                device = adapter.getRemoteDevice(mac)
            } catch (e: Exception) {
                onResult(false, "Invalid Bluetooth MAC address: ${e.message}")
                return@execute
            }

            val targetUuid = try {
                if (!uuidStr.isNullOrEmpty()) UUID.fromString(uuidStr) else SPP_UUID
            } catch (_: Exception) {
                SPP_UUID
            }

            var currentSocket: BluetoothSocket? = null

            // Attempt 1: Standard Secure RFCOMM
            try {
                currentSocket = device.createRfcommSocketToServiceRecord(targetUuid)
                currentSocket.connect()
            } catch (e1: Exception) {
                try {
                    currentSocket?.close()
                } catch (_: Exception) {}

                // Attempt 2: Insecure RFCOMM
                try {
                    currentSocket = device.createInsecureRfcommSocketToServiceRecord(targetUuid)
                    currentSocket.connect()
                } catch (e2: Exception) {
                    try {
                        currentSocket?.close()
                    } catch (_: Exception) {}

                    // Attempt 3: Channel 1 reflection fallback
                    try {
                        val m = device.javaClass.getMethod("createRfcommSocket", Int::class.javaPrimitiveType)
                        currentSocket = m.invoke(device, 1) as BluetoothSocket
                        currentSocket.connect()
                    } catch (e3: Exception) {
                        try {
                            currentSocket?.close()
                        } catch (_: Exception) {}
                        onResult(false, "Failed to connect RFCOMM: ${e3.message ?: e2.message}")
                        return@execute
                    }
                }
            }

            try {
                socket = currentSocket
                inputStream = currentSocket.inputStream
                outputStream = currentSocket.outputStream
                isConnected = true
                connectedMac = mac
                connectedName = device.name ?: mac

                startReaderThread()

                val params = Arguments.createMap()
                params.putBoolean("connected", true)
                params.putString("mac", mac)
                params.putString("name", connectedName)
                eventEmitter("onBtConnectionChange", params)

                onResult(true, null)
            } catch (e: Exception) {
                disconnectInternal(notify = true)
                onResult(false, "Socket stream setup failed: ${e.message}")
            }
        }
    }

    private fun startReaderThread() {
        readerThread?.interrupt()
        readerThread = Thread({
            val stream = inputStream ?: return@Thread
            val buffer = ByteArray(4096)
            val lineBuilder = StringBuilder()

            while (isConnected && !Thread.currentThread().isInterrupted) {
                try {
                    val bytesRead = stream.read(buffer)
                    if (bytesRead <= 0) {
                        break
                    }
                    val text = String(buffer, 0, bytesRead, Charsets.UTF_8)
                    lineBuilder.append(text)

                    while (lineBuilder.contains("\n")) {
                        val newlineIndex = lineBuilder.indexOf("\n")
                        val line = lineBuilder.substring(0, newlineIndex).trim()
                        lineBuilder.delete(0, newlineIndex + 1)

                        if (line.isNotEmpty()) {
                            val msgMap = Arguments.createMap()
                            msgMap.putString("message", line)
                            eventEmitter("onBtMessage", msgMap)
                        }
                    }
                } catch (e: Exception) {
                    break
                }
            }

            if (isConnected) {
                disconnectInternal(notify = true)
            }
        }, "BtSppReaderThread").apply {
            isDaemon = true
            start()
        }
    }

    @Synchronized
    fun send(message: String): Boolean {
        val stream = outputStream ?: return false
        return try {
            val formatted = if (message.endsWith("\n")) message else "$message\n"
            stream.write(formatted.toByteArray(Charsets.UTF_8))
            stream.flush()
            true
        } catch (e: Exception) {
            disconnectInternal(notify = true)
            false
        }
    }

    fun disconnect() {
        executor.execute {
            disconnectInternal(notify = true)
        }
    }

    @Synchronized
    private fun disconnectInternal(notify: Boolean) {
        if (!isConnected && socket == null) return

        isConnected = false
        val oldMac = connectedMac
        val oldName = connectedName
        connectedMac = null
        connectedName = null

        try {
            readerThread?.interrupt()
        } catch (_: Exception) {}
        readerThread = null

        try {
            inputStream?.close()
        } catch (_: Exception) {}
        inputStream = null

        try {
            outputStream?.close()
        } catch (_: Exception) {}
        outputStream = null

        try {
            socket?.close()
        } catch (_: Exception) {}
        socket = null

        if (notify) {
            val params = Arguments.createMap()
            params.putBoolean("connected", false)
            params.putString("mac", oldMac)
            params.putString("name", oldName)
            eventEmitter("onBtConnectionChange", params)
        }
    }
}
