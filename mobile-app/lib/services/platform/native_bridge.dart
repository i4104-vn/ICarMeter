import 'dart:async';
import 'package:flutter/services.dart';

/// Platform channel bridge to Android/iOS native layer for Bluetooth SPP, Wi-Fi binding, and GPS service.
class NativeBridge {
  static const MethodChannel _channel = MethodChannel('com.icarmeter.companion/bridge');
  static const EventChannel _btEventChannel = EventChannel('com.icarmeter.companion/bt_events');
  static const EventChannel _locationEventChannel = EventChannel('com.icarmeter.companion/location_events');
  static const EventChannel _wifiEventChannel = EventChannel('com.icarmeter.companion/wifi_events');

  static Stream<Map<String, dynamic>>? _btEventStream;
  static Stream<Map<String, dynamic>>? _locationEventStream;
  static Stream<Map<String, dynamic>>? _wifiEventStream;

  /// Broadcast stream of Bluetooth connection events, state changes, and received raw messages.
  static Stream<Map<String, dynamic>> get btEvents {
    _btEventStream ??= _btEventChannel
        .receiveBroadcastStream()
        .map((event) => Map<String, dynamic>.from(event as Map));
    return _btEventStream!;
  }

  /// Broadcast stream of device location updates providing geographic speed in m/s.
  static Stream<Map<String, dynamic>> get locationEvents {
    _locationEventStream ??= _locationEventChannel
        .receiveBroadcastStream()
        .map((event) => Map<String, dynamic>.from(event as Map));
    return _locationEventStream!;
  }

  /// Broadcast stream of system Wi-Fi connectivity and hotspot status changes.
  static Stream<Map<String, dynamic>> get wifiEvents {
    _wifiEventStream ??= _wifiEventChannel
        .receiveBroadcastStream()
        .map((event) => Map<String, dynamic>.from(event as Map));
    return _wifiEventStream!;
  }

  /// Connects to a target Bluetooth device with [mac] address via RFCOMM / SPP.
  ///
  /// Optionally accepts a custom service [uuid]. Returns `true` if connection succeeds.
  static Future<bool> connectBtSpp(String mac, {String? uuid}) async {
    try {
      final ok = await _channel.invokeMethod<bool>('connectBtSpp', {
        'mac': mac,
        'uuid': uuid,
      });
      return ok ?? false;
    } catch (_) {
      return false;
    }
  }

  /// Closes the active Bluetooth SPP RFCOMM socket connection.
  static Future<bool> disconnectBtSpp() async {
    try {
      final ok = await _channel.invokeMethod<bool>('disconnectBtSpp');
      return ok ?? false;
    } catch (_) {
      return false;
    }
  }

  /// Transmits a raw string [message] across the active Bluetooth SPP socket.
  static Future<bool> btSend(String message) async {
    try {
      final ok = await _channel.invokeMethod<bool>('btSend', {'message': message});
      return ok ?? false;
    } catch (_) {
      return false;
    }
  }

  /// Queries the platform layer to verify whether Bluetooth SPP is currently connected.
  static Future<bool> isBtSppConnected() async {
    try {
      final ok = await _channel.invokeMethod<bool>('isBtSppConnected');
      return ok ?? false;
    } catch (_) {
      return false;
    }
  }

  /// Instructs Android OS to route app network traffic through the vehicle's Wi-Fi hotspot.
  static Future<bool> bindToCarWifi() async {
    try {
      final ok = await _channel.invokeMethod<bool>('bindToCarWifi');
      return ok ?? false;
    } catch (_) {
      return false;
    }
  }

  /// Activates the native GPS location listener with the specified sampling [rateHz].
  static Future<bool> startGps({int rateHz = 2}) async {
    try {
      final ok = await _channel.invokeMethod<bool>('startGps', {'rateHz': rateHz});
      return ok ?? false;
    } catch (_) {
      return false;
    }
  }

  /// Deactivates the native GPS location listener.
  static Future<bool> stopGps() async {
    try {
      final ok = await _channel.invokeMethod<bool>('stopGps');
      return ok ?? false;
    } catch (_) {
      return false;
    }
  }

  /// Starts the persistent Android foreground service with a persistent notification.
  static Future<bool> startForegroundService() async {
    try {
      final ok = await _channel.invokeMethod<bool>('startForegroundService');
      return ok ?? false;
    } catch (_) {
      return false;
    }
  }

  /// Terminates the persistent Android foreground service.
  static Future<bool> stopForegroundService() async {
    try {
      final ok = await _channel.invokeMethod<bool>('stopForegroundService');
      return ok ?? false;
    } catch (_) {
      return false;
    }
  }
}
