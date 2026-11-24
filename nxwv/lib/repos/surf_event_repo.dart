import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nxwv/models/surf_event_model.dart';

// abstract class SurfEventRepo {
//   Stream<List<SurfEvent>> watchForecasts();
// }

/// Current implementation: reads assets/data/.json and emits it as a stream.
/// Change `pollInterval` to `null` if you only want a single emission on load.
class PsuedoSurfEventRepo {
  PsuedoSurfEventRepo({
    this.assetPath = 'assets/data/local_results_v2.json',
    this.pollInterval = const Duration(seconds: 3),
  });

  final String assetPath;
  final Duration? pollInterval;

  Future<List<SurfEvent>> _loadOnce() async {
    final raw = await rootBundle.loadString(assetPath);
    final decoded = jsonDecode(raw) as List<dynamic>;
    return decoded
        .map((item) => SurfEvent.fromJson(item as Map<String, dynamic>))
        .toList();
  }

  Stream<List<SurfEvent>> watchForecasts() async* {
    yield await _loadOnce();

    if (pollInterval != null) {
      await for (final _ in Stream.periodic(pollInterval!)) {
        yield await _loadOnce();
      }
    }
  }
}

/// Future Firestore implementation:
/// class SurfEventRepo implements SurfEventRepo {
///   SurfEventRepo(this._firestore);
///   final FirebaseFirestore _firestore;
///
///   @override
///   Stream<List<SurfEvent>> watchForecasts() {
///     return _firestore
///         .collection('surf_forecasts') // your collection name
///         .snapshots()
///         .map((snap) => snap.docs.map((d) => SurfEvent.fromJson(d.data())).toList());
///   }
/// }

final surfEventRepoProvider = Provider<PsuedoSurfEventRepo>(
  isAutoDispose: true,
  (ref) => PsuedoSurfEventRepo(),
);

// final surfEventRepoProvider = Provider<SurfEventRepo>(
//   isAutoDispose: true,
//   (ref) => SurfEventRepo(),
// );
