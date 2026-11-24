import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nxwv/models/surf_event_model.dart';
import 'package:nxwv/repos/surf_event_repo.dart';
import 'package:nxwv/surf_events/notifiers/surf_event_stream_notifier.dart';

/*

From SurfEvents, get a distinct list of dates

*/

class SurfDatesSetNotifier extends Notifier<List<DateTime>> {
  @override
  List<DateTime> build() {
    final surfEventAsync = ref.watch(surfEventsStreamProvider);
    if (!surfEventAsync.hasValue) {
      return [];
    } else {
      final List<SurfEvent> surfEvents = surfEventAsync.value!;
      final Set<DateTime> dateSet = {};
      for (var event in surfEvents) {
        final startDate = event.start;
        dateSet.add(startDate);
      }
      final dateList = dateSet.toList();
      return dateList;
    }
  }
}

final surfDatesSetProvider =
    NotifierProvider<SurfDatesSetNotifier, List<DateTime>>(
      isAutoDispose: false,
      SurfDatesSetNotifier.new,
    );
