import 'dart:ui';

import 'package:nxwv/constants.dart';

/*
firestore data model ideas:
just a SPOT model... so a doc per spot
  THEN SPOT has a SurfEvents var which is a list of dict (SurfEvent)
*/

enum Surfability {
  surfschool('Surf School Slop', 1, kNwPeri),
  surfable('Surfable', 2, kNwLBlue),
  works('Works', 3, kNwSurf),
  on('ON!', 4, kNwMint);

  const Surfability(this.name, this.rank, this.color);
  final String name;
  final int rank;
  final Color color;

  static Surfability fromPeakScore(double score) {
    if (score < 35.0) return Surfability.surfschool;
    if (score < 55.0) return Surfability.surfable;
    if (score < 75.0) return Surfability.works;
    return Surfability.on;
  }
}

class SurfEvent {
  const SurfEvent({
    required this.spot,
    required this.start,
    required this.end,
    required this.peakTime,
    required this.peakScore,
    required this.meanScore,
    required this.label,
    required this.surfableHours,
  });

  final String spot;
  final DateTime start;
  final DateTime end;
  final DateTime peakTime;
  final double peakScore;
  final double meanScore;
  final Surfability label;
  final int surfableHours;

  factory SurfEvent.fromJson(Map<String, dynamic> json) {
    return SurfEvent(
      spot: json['spot'] as String,
      start: DateTime.parse(json['start'] as String),
      end: DateTime.parse(json['end'] as String),
      peakTime: DateTime.parse(json['peak_time'] as String),
      peakScore: (json['peak_score'] as num).toDouble(),
      meanScore: (json['mean_score'] as num).toDouble(),
      // label: json['label'] as String,
      label: Surfability.fromPeakScore((json['peak_score'] as num).toDouble()),
      surfableHours: (json['surfable_hours'] as num).toInt(),
    );
  }
}
