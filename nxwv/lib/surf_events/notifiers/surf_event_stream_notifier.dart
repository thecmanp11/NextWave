import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nxwv/models/surf_event_model.dart';
import 'package:nxwv/repos/surf_event_repo.dart';

class SurfEventsStreamNotifier extends StreamNotifier<List<SurfEvent>> {
  @override
  Stream<List<SurfEvent>> build() {
    final repo = ref.watch(surfEventRepoProvider);
    final Stream<List<SurfEvent>> surfStream = repo.watchForecasts();
    return surfStream;
  }
}

final surfEventsStreamProvider =
    StreamNotifierProvider<SurfEventsStreamNotifier, List<SurfEvent>>(
      isAutoDispose: false,
      SurfEventsStreamNotifier.new,
    );
