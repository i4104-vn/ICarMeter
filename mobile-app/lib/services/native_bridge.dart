import 'dart:async';
import 'package:flutter/services.dart';

class NativeBridge {
  static const MethodChannel _channel = MethodChannel('com.icarmeter.companion/bridge');
  static const EventChannel _btEventChannel = EventChannel('com.icarmeter.companion/bt_events');
  static const EventChannel _locationEventChannel = EventChannel('com.icarmeter.companion/location_events');
  static const EventChannel _wifiEventChannel = EventChannel('com.icarmeter.companion/wifi_events');

  static Stream<Map<String, dynamic>>? _btEventStream;
  static Stream<Map<String, dynamic>>? _locationEventStream;
  static Stream<Map<String, dynamic>>? _wifiEventStream;

  static Stream<Map<String, dynamic>> get btEvents {
    _btEventStream ??= _btEventChannel
        .receiveBroadcastStream()
        .map((event) => Map<String, dynamic>.from(event as Map));
    return _btEventStream!;
  }

  static Stream<Map<String, dynamic>> get locationEvents {
    _locationEventStream ??= _locationEventChannel
        .receiveBroadcastStream()
        .map((event) => Map<String, dynamic>.from(event as Map));
    return _locationEventStream!;
  }

  static Stream<Map<String, dynamic>> get wifiEvents {
    _wifiEventStream ??= _wifiEventChannel
        .receiveBroadcastStream()
        .map((event) => Map<String, dynamic>.from(event as Map));
    return _wifiEventStream!;
  }

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

  static Future<bool> disconnectBtSpp() async {
    try {
      final ok = await _channel.invokeMethod<bool>('disconnectBtSpp');
      return ok ?? false;
    } catch (_) {
      return false;
    }
  }

  static Future<bool> btSend(String message) async {
    try {
      final ok = await _channel.invokeMethod<bool>('btSend', {'message': message});
      return ok ?? false;
    } catch (_) {
      return false;
    }
  }

  static Future<bool> isBtSppConnected() async {
    try {
      final ok = await _channel.invokeMethod<bool>('isBtSppConnected');
      return ok ?? false;
    } catch (_) {
      return false;
    }
  }

  static Future<bool> bindToCarWifi() async {
    try {
      final ok = await _channel.invokeMethod<bool>('bindToCarWifi');
      return ok ?? false;
    } catch (_) {
      return false;
    }
  }

  static Future<bool> startGps({int rateHz = 2}) async {
    try {
      final ok = await _channel.invokeMethod<bool>('startGps', {'rateHz': rateHz});
      return ok ?? false;
    } catch (_) {
      return false;
    }
  }

  static Future<bool> stopGps() async {
    try {
      final ok = await _channel.invokeMethod<bool>('stopGps');
      return ok ?? false;
    } catch (_) {
      return false;
    }
  }

  static Future<bool> startForegroundService() async {
    try {
      final ok = await _channel.invokeMethod<bool>('startForegroundService');
      return ok ?? false;
    } catch (_) {
      return false;
    }
  }

  static Future<bool> stopForegroundService() async {
    try {
      final ok = await _channel.invokeMethod<bool>('stopForegroundService');
      return ok ?? false;
    } catch (_) {
      return false;
    }
  }
}
