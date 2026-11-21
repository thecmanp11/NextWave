import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nxwv/constants.dart';
import 'package:nxwv/models/surf_event_model.dart';
import 'package:nxwv/surf_events/notifiers/surf_event_stream_notifier.dart';
import 'package:nxwv/surf_events/ui/surf_event_groups.dart';
import 'package:nxwv/widgets/flat.dart';

class SurfEventScroll extends ConsumerWidget {
  const new({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final AsyncValue<List<SurfEvent>> asyncSurfEvents = ref.watch(
      surfEventsStreamProvider,
    );

    return Expanded(
      // TODO: change to grid view
      child: CustomScrollView(
        shrinkWrap: true,
        scrollDirection: Axis.vertical,
        // physics: kNormalBouncePhysics,
        slivers: [
          asyncSurfEvents.when(
            loading: () => const SliverFillRemaining(
              hasScrollBody: false,
              child: Center(child: CircularProgressIndicator(color: kNwMint)),
            ),
            data: (surfEvents) {
              final groupedDates = groupSurfEvents(surfEvents);

              if (surfEvents.isEmpty) {
                return SliverFillRemaining(
                  hasScrollBody: false,
                  child: Center(child: FlatTruck()),
                );
              }

              // Group Dates
              return SliverPadding(
                padding: const EdgeInsets.symmetric(
                  horizontal: 16,
                  vertical: 12,
                ),
                sliver: SliverList.builder(
                  itemCount: groupedDates.length,
                  itemBuilder: (context, index) {
                    // final event = surfEvents[index];
                    return DateSectionCard(group: groupedDates[index]);
                  },
                ),
              );
            },
            error: (e, st) {
              print('SurfEventScroll: $e - $st');

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
        ],
      ),
    );
  }
}
