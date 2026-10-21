import 'dart:async';
import 'package:flutter/foundation.dart';
import '../../models/trip.dart';
import '../storage/storage_service.dart';

/// Service managing trip recording, distance accumulation, and idle timeouts.
class TripTracker extends ChangeNotifier {
  final StorageService storage;
  List<TripRecord> _trips = [];
  TripRecord? _activeTrip;
  Timer? _tripIdleTimer;

  /// Initializes the trip tracker and loads previously stored trips from [storage].
  TripTracker(this.storage) {
    _trips = storage.getTrips();
  }

  /// An unmodifiable view of all completed trip records.
  List<TripRecord> get trips => List.unmodifiable(_trips);

  /// The currently active in-progress trip, or `null` if the vehicle is idle.
  TripRecord? get activeTrip => _activeTrip;

  /// Updates trip metrics based on current [speedKmh].
  ///
  /// - Starts a new trip record if speed exceeds `5.0 km/h` and no trip is active.
  /// - Accumulates distance, updates duration, and recalculates average and max
  ///   speed while the vehicle is in motion.
  /// - Starts a 3-minute idle timer when the vehicle comes to a stop.
  void updateSpeed(double speedKmh) {
    final isMoving = speedKmh > 5.0;
    final hasActiveTrip = _activeTrip != null;

    switch ((isMoving, hasActiveTrip)) {
      case (true, false):
        _tripIdleTimer?.cancel();
        _tripIdleTimer = null;
        _activeTrip = TripRecord(
          id: DateTime.now().millisecondsSinceEpoch.toString(),
          startTime: DateTime.now(),
        );
        notifyListeners();
        break;

      case (true, true):
        _tripIdleTimer?.cancel();
        _tripIdleTimer = null;
        final now = DateTime.now();
        final trip = _activeTrip!;
        final durationSec = now.difference(trip.startTime).inSeconds;
        final maxSpeed = speedKmh > trip.maxSpeedKmh ? speedKmh : trip.maxSpeedKmh;
        final distanceDelta = (speedKmh / 3600.0);
        final newDistance = trip.distanceKm + distanceDelta;
        final avgSpeed = durationSec > 0 ? (newDistance / (durationSec / 3600.0)) : speedKmh;

        _activeTrip = trip.copyWith(
          durationSeconds: durationSec,
          distanceKm: newDistance,
          maxSpeedKmh: maxSpeed,
          avgSpeedKmh: avgSpeed,
        );
        notifyListeners();
        break;

      case (false, true):
        _tripIdleTimer ??= Timer(const Duration(minutes: 3), () {
          finishActiveTrip();
        });
        break;

      case (false, false):
        break;
    }
  }

  /// Concludes the active trip, sets the end timestamp, and persists it to [storage].
  ///
  /// Trips shorter than `0.1 km` are discarded to avoid saving false starts.
  void finishActiveTrip() {
    if (_activeTrip == null) return;
    final finished = _activeTrip!.copyWith(endTime: DateTime.now());
    if (finished.distanceKm > 0.1) {
      _trips.insert(0, finished);
      storage.saveTrips(_trips);
    }
    _activeTrip = null;
    _tripIdleTimer?.cancel();
    _tripIdleTimer = null;
    notifyListeners();
  }

  /// Removes all historical trips from memory and empties persisted storage.
  void clearTrips() {
    _trips.clear();
    storage.saveTrips([]);
    notifyListeners();
  }

  /// Cancels active idle timers and cleans up resources.
  @override
  void dispose() {
    _tripIdleTimer?.cancel();
    super.dispose();
  }
}
