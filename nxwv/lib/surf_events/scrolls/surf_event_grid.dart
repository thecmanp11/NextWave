import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nxwv/constants.dart';
import 'package:nxwv/models/surf_event_model.dart';
import 'package:nxwv/surf_events/notifiers/surf_dates_set_notifier.dart';
import 'package:nxwv/surf_events/notifiers/surf_event_stream_notifier.dart';
import 'package:nxwv/surf_events/ui/surf_event_grid_cell.dart';
import 'package:nxwv/widgets/flat.dart';

class SurfEventGrid extends ConsumerWidget {
  const new({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final AsyncValue<List<SurfEvent>> asyncSurfEvents = ref.watch(
      surfEventsStreamProvider,
    );

    return Expanded(
      child: asyncSurfEvents.when(
        loading: () => const Center(
          child: CircularProgressIndicator(color: kNwMint),
        ),
        data: (surfEvents) {
          // NO incoming swell
          if (surfEvents.isEmpty) {
            return Center(child: FlatTruck());
          }

          // dateSet
          final dateSet = ref.watch(surfDatesSetProvider);

          // surfEvents.sort((a, b) => a.start.compareTo(b.start));
          // final sortedEvents = List<SurfEvent>.from(surfEvents)
          //   ..sort((a, b) => a.start.compareTo(b.start));
          // TODO: sort again within date by quality

          // build sliver based on DATES -- then look UP events based on dates, sort them there
          return CustomScrollView(
            slivers: [
              SliverList.builder(
                itemCount: dateSet.length,
                itemBuilder: (context, index) {
                  final surfDate = dateSet[index];

                  final sortedEvents =
                      surfEvents
                          .where(
                            (event) =>
                                event.start.year == surfDate.year &&
                                event.start.month == surfDate.month &&
                                event.start.day == surfDate.day,
                          )
                          .toList()
                        ..sort((a, b) => b.meanScore.compareTo(a.meanScore));

                  if (sortedEvents.isEmpty) {
                    return const SizedBox.shrink();
                  }

                  // avg score for ALL spots this day
                  Surfability _averageDayHourlyScores(List<SurfEvent> events) {
                    var total = 0.0;
                    var count = 0;

                    for (final event in events) {
                      final scores = event.hourlyScores?.values;
                      if (scores == null || scores.isEmpty) continue;

                      for (final score in scores) {
                        total += score;
                        count++;
                      }
                    }

                    if (count == 0) {
                      if (events.isEmpty) return Surfability.surfschool;
                      final meanSum = events.fold<double>(
                        0.0,
                        (sum, e) => sum + e.meanScore,
                      );
                      final avg = meanSum / events.length;
                      return Surfability.fromPeakScore(avg);
                    }

                    final avg = total / count;
                    return Surfability.fromPeakScore(avg);
                  }

                  final meanDaySurfability = _averageDayHourlyScores(
                    sortedEvents,
                  );

                  return Padding(
                    padding: const EdgeInsets.only(bottom: 16),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        // Date Header
                        Padding(
                          padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
                          child: Text(
                            _formatDateHeader(surfDate),
                            style: NwTextStyles.title.c(
                              // kNwLBlue.withValues(alpha: 0.3),
                              meanDaySurfability.color.withValues(alpha: 0.3),
                            ),
                          ),
                        ),

                        // Grid of SurfEvents for that date
                        LayoutBuilder(
                          builder: (context, constraints) {
                            const double horizontalPadding = 12.0;
                            const double cardW = 240;
                            const double minGap = 10.0;
                            const double maxGap =
                                28.0; // Tolerance before adding another column

                            final availWidth =
                                (constraints.maxWidth - horizontalPadding * 2)
                                    .clamp(
                                      cardW,
                                      double.infinity,
                                    );

                            // How many fixed-width cards fit at minimum spacing
                            final int cols = math.max(
                              1,
                              ((availWidth + minGap) / (cardW + minGap))
                                  .floor(),
                            );

                            // Distribute extra horizontal width between cards, capped at maxGap tolerance
                            final double gap = cols > 1
                                ? ((availWidth - cols * cardW) / (cols - 1))
                                      .clamp(minGap, maxGap)
                                : minGap;

                            return Padding(
                              padding: EdgeInsets.symmetric(
                                horizontal: horizontalPadding,
                              ),
                              child: Center(
                                child: Wrap(
                                  spacing: gap,
                                  runSpacing: 12.0,
                                  alignment: WrapAlignment.start,
                                  children: [
                                    for (final event in sortedEvents)
                                      SurfEventGridCell(
                                        surfEvent: event,
                                        showDate: false,
                                      ),
                                  ],
                                ),
                              ),
                            );
                          },
                        ),
                      ],
                    ),
                  );
                },
              ),
            ],
          );
        },

        error: (e, st) {
          print('SurfEventGrid: $e - $st');
          return Center(
            child: Text(
              'Error loading surf events: $e',
              style: NwTextStyles.bodySmall.c(kNwFlatRed),
            ),
          );
        },
      ),
    );
  }

  String _formatDateHeader(DateTime dt) {
    const weekdays = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
    const months = [
      'Jan',
      'Feb',
      'Mar',
      'Apr',
      'May',
      'Jun',
      'Jul',
      'Aug',
      'Sep',
      'Oct',
      'Nov',
      'Dec',
    ];
    final weekday = weekdays[dt.weekday - 1];
    final month = months[dt.month - 1];
    return '$weekday. $month ${dt.day}, ${dt.year}';
  }
}
