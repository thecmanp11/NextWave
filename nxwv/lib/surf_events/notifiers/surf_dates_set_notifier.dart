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
    }

    // final spotsMap = ref.watch(surfSpotProvider).value ?? const {};
    final surfEvents = surfEventAsync.value!;
    final Set<DateTime> dateSet = {};

    for (final event in surfEvents) {
      // Convert to the spot's timezone first, then strip hours/minutes/seconds
      // final spotDt = toSpotTime(event.start, spotsMap[event.spot]?.timezone);
      // dateSet.add(DateTime(spotDt.year, spotDt.month, spotDt.day));
      dateSet.add(
        DateTime(event.start.year, event.start.month, event.start.day),
      );
    }

    return dateSet.toList()..sort((a, b) => a.compareTo(b));
  }
}

// class SurfDatesSetNotifier extends Notifier<List<DateTime>> {
//   @override
//   List<DateTime> build() {
//     final surfEventAsync = ref.watch(surfEventsStreamProvider);
//     if (!surfEventAsync.hasValue) {
//       return [];
//     } else {
//       final List<SurfEvent> surfEvents = surfEventAsync.value!;
//       final Set<DateTime> dateSet = {};
//       for (var event in surfEvents) {
//         final startDate = event.start;
//         dateSet.add(startDate);
//       }
//       final dateList = dateSet.toList();
//       return dateList;
//     }
//   }
// }

final surfDatesSetProvider =
    NotifierProvider<SurfDatesSetNotifier, List<DateTime>>(
      isAutoDispose: false,
      SurfDatesSetNotifier.new,
    );
