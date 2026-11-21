import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class SurfSpot {
  const SurfSpot({
    required this.id,
    required this.name,
    required this.region,
    required this.lat,
    required this.lon,
    required this.facing,
    required this.window,
    required this.idealSwellDir,
    required this.minHs,
    required this.maxHs,
    required this.minPeriod,
    required this.idealPeriod,
    required this.minPowerKwM,
    required this.idealPowerKwM,
    required this.offshoreDir,
    required this.maxOnshoreKn,
    required this.maxOffshoreKn,
    required this.breakType,
    required this.notes,
    this.useWindWaves = false,
    this.driverWindDir,
    this.minDriverWindKn,
    this.seasonMonths = const [],
  });

  final String id;
  final String name;
  final String region;
  final double lat;
  final double lon;
  final double facing;
  final double window;
  final List<double> idealSwellDir;
  final double minHs;
  final double maxHs;
  final double minPeriod;
  final List<double> idealPeriod;
  final double minPowerKwM;
  final double idealPowerKwM;
  final List<double> offshoreDir;
  final double maxOnshoreKn;
  final double maxOffshoreKn;
  final String breakType;
  final String notes;
  final bool useWindWaves;
  final List<double>? driverWindDir;
  final double? minDriverWindKn;
  final List<int> seasonMonths;

  static List<double> _toDoubleList(dynamic raw) {
    if (raw is! List) return const [];
    return raw.map((e) => (e as num).toDouble()).toList();
  }

  static List<int> _toIntList(dynamic raw) {
    if (raw is! List) return const [];
    return raw.map((e) => (e as num).toInt()).toList();
  }

  factory SurfSpot.fromJson(String id, Map<String, dynamic> json) {
    return SurfSpot(
      id: id,
      name: json['name'] as String,
      region: json['region'] as String,
      lat: (json['lat'] as num).toDouble(),
      lon: (json['lon'] as num).toDouble(),
      facing: (json['facing'] as num).toDouble(),
      window: (json['window'] as num).toDouble(),
      idealSwellDir: _toDoubleList(json['ideal_swell_dir']),
      minHs: (json['min_hs'] as num).toDouble(),
      maxHs: (json['max_hs'] as num).toDouble(),
      minPeriod: (json['min_period'] as num).toDouble(),
      idealPeriod: _toDoubleList(json['ideal_period']),
      minPowerKwM: (json['min_power_kw_m'] as num).toDouble(),
      idealPowerKwM: (json['ideal_power_kw_m'] as num).toDouble(),
      offshoreDir: _toDoubleList(json['offshore_dir']),
      maxOnshoreKn: (json['max_onshore_kn'] as num).toDouble(),
      maxOffshoreKn: (json['max_offshore_kn'] as num).toDouble(),
      breakType: json['break_type'] as String,
      notes: json['notes'] as String? ?? '',
      useWindWaves: json['use_wind_waves'] as bool? ?? false,
      driverWindDir: json['driver_wind_dir'] != null
          ? _toDoubleList(json['driver_wind_dir'])
          : null,
      minDriverWindKn: json['min_driver_wind_kn'] != null
          ? (json['min_driver_wind_kn'] as num).toDouble()
          : null,
      seasonMonths: _toIntList(json['season_months']),
    );
  }
}

// Surf Spot Provider
class SurfSpotNotifier extends AsyncNotifier<Map<String, SurfSpot>> {
  @override
  Future<Map<String, SurfSpot>> build() async {
    final raw = await rootBundle.loadString('assets/data/spots.json');
    final decoded = jsonDecode(raw) as Map<String, dynamic>;
    return decoded.map(
      (key, value) =>
          MapEntry(key, SurfSpot.fromJson(key, value as Map<String, dynamic>)),
    );
  }
}

final surfSpotProvider =
    AsyncNotifierProvider<SurfSpotNotifier, Map<String, SurfSpot>>(
      isAutoDispose: true,
      SurfSpotNotifier.new,
    );
