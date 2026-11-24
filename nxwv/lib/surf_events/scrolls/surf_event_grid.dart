import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nxwv/constants.dart';
import 'package:nxwv/models/surf_event_model.dart';
import 'package:nxwv/surf_events/notifiers/surf_event_stream_notifier.dart';
import 'package:nxwv/surf_events/ui/surf_event_grid_cell.dart';
import 'package:nxwv/surf_events/ui/surf_event_groups.dart';
import 'package:nxwv/widgets/flat.dart';

class SurfEventGrid extends ConsumerWidget {
  const new({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final AsyncValue<List<SurfEvent>> asyncSurfEvents = ref.watch(
      surfEventsStreamProvider,
    );

    return Expanded(
      // TODO: change to grid view
      child: asyncSurfEvents.when(
        loading: () => const SliverFillRemaining(
          hasScrollBody: false,
          child: Center(child: CircularProgressIndicator(color: kNwMint)),
        ),
        data: (surfEvents) {
          if (surfEvents.isEmpty) {
            return SliverFillRemaining(
              hasScrollBody: false,
              child: Center(child: FlatTruck()),
            );
          }
          // surfEvents.sort((a, b) => a.start.compareTo(b.start));
          final sortedEvents = List<SurfEvent>.from(surfEvents)
            ..sort((a, b) => a.start.compareTo(b.start));

          return GridView.builder(
            padding: const EdgeInsets.all(12),
            gridDelegate: SliverGridDelegateWithMaxCrossAxisExtent(
              maxCrossAxisExtent: 240,
              mainAxisExtent: 180,
              mainAxisSpacing: 12,
              crossAxisSpacing: 12,
            ),
            itemCount: sortedEvents.length,
            itemBuilder: (context, index) {
              final surfEvent = sortedEvents[index];
              final bool showDate =
                  index == 0 ||
                  !_isSameCalendarDay(
                    surfEvents[index - 1].start,
                    surfEvent.start,
                  );

              // return Text('${surfEvent.spot}');
              return SurfEventGridCell(
                surfEvent: surfEvent,
                showDate: showDate,
              );
            },
          );
        },
        error: (e, st) {
          print('SurfEventGrid: $e - $st');

          return SliverFillRemaining(
            hasScrollBody: false,
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Text(
                'Error loading surf events: $e',
                style: NwTextStyles.bodySmall.c(kNwFlatRed),
              ),
            ),
          );
        },
      ),
    );
  }

  bool _isSameCalendarDay(DateTime a, DateTime b) {
    final da = DateUtils.dateOnly(a.toLocal());
    final db = DateUtils.dateOnly(b.toLocal());
    return da.year == db.year && da.month == db.month && da.day == db.day;
  }
}
