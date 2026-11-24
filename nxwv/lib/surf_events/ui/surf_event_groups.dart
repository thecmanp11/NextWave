import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nxwv/constants.dart';
import 'package:nxwv/models/spot_model.dart';
import 'package:nxwv/models/surf_event_model.dart';
import 'package:nxwv/widgets/hourly_score_bar_chart.dart';
import 'package:nxwv/widgets/regional_icon.dart';

class SpotEventsGroup {
  const SpotEventsGroup({
    required this.spot,
    required this.maxMeanScore,
    required this.events,
  });

  final String spot;
  final double maxMeanScore;
  final List<SurfEvent> events; // Sorted by start time (soonest first)
}

class DateSurfEventsGroup {
  const DateSurfEventsGroup({
    required this.date, // Normalized to midnight (year, month, day)
    required this.spotGroups,
  });

  final DateTime date;
  final List<SpotEventsGroup> spotGroups; // Sorted by highest meanScore first
}

List<DateSurfEventsGroup> groupSurfEvents(List<SurfEvent> events) {
  if (events.isEmpty) return const [];

  // 1. Group all events by calendar date (normalized to local midnight)
  final Map<DateTime, List<SurfEvent>> byDate = {};
  for (final event in events) {
    final localStart = event.start.toLocal();
    final dateKey = DateTime(localStart.year, localStart.month, localStart.day);
    byDate.putIfAbsent(dateKey, () => []).add(event);
  }

  // Sort dates soonest -> latest
  final sortedDates = byDate.keys.toList()..sort((a, b) => a.compareTo(b));

  return sortedDates.map((dateKey) {
    final dayEvents = byDate[dateKey]!;

    // 2. Within this date, group events by spot
    final Map<String, List<SurfEvent>> bySpot = {};
    for (final event in dayEvents) {
      bySpot.putIfAbsent(event.spot, () => []).add(event);
    }

    final spotGroups =
        bySpot.entries.map((entry) {
            final spot = entry.key;
            final spotEvents = List<SurfEvent>.from(entry.value)
              // 3. Within this spot, sort events by start time (soonest first)
              ..sort((a, b) => a.start.compareTo(b.start));

            // Highest mean_score for this spot on this date
            final maxMeanScore = spotEvents
                .map((e) => e.meanScore)
                .reduce((a, b) => a > b ? a : b);

            return SpotEventsGroup(
              spot: spot,
              maxMeanScore: maxMeanScore,
              events: spotEvents,
            );
          }).toList()
          // Sort spots by highest mean_score first (ties broken alphabetically by spot)
          ..sort((a, b) {
            final scoreCmp = b.maxMeanScore.compareTo(a.maxMeanScore);
            if (scoreCmp != 0) return scoreCmp;
            return a.spot.compareTo(b.spot);
          });

    return DateSurfEventsGroup(date: dateKey, spotGroups: spotGroups);
  }).toList();
}

class DateSectionCard extends ConsumerWidget {
  const DateSectionCard({super.key, required this.group});

  final DateSurfEventsGroup group;

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

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // surf spot info
    final spotsMap = ref.watch(surfSpotProvider).value ?? const {};

    return Column(
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        // Level 1: Date Header
        SizedBox(
          width: double.infinity,
          child: Align(
            alignment: const Alignment(-0.20, 0.0), // X% left of center
            child: Padding(
              padding: const EdgeInsets.symmetric(vertical: 18, horizontal: 16),
              child: Text(
                _formatDateHeader(group.date),
                style: NwTextStyles.title,
                textAlign: TextAlign.left,
              ),
            ),
          ),
        ),

        // Level 2 & 3: Spots (by highest meanScore) -> Event times (soonest first)
        ...group.spotGroups.map(
          (spotGroup) => SpotSubSection(
            spotGroup: spotGroup,
            spotInfo: spotsMap[spotGroup.spot],
          ),
        ),

        SizedBox(height: 44),
      ],
    );
  }
}

class SpotSubSection extends StatelessWidget {
  const SpotSubSection({super.key, required this.spotGroup, this.spotInfo});
  final SpotEventsGroup spotGroup;
  final SurfSpot? spotInfo;

  // TODO: don't want to localize time to MY location... I want the times for the SPOT
  String _formatTimeWindow(DateTime startUtc, DateTime endUtc) {
    final start = startUtc.toLocal();
    final end = endUtc.toLocal();
    String twoDigits(int n) => n.toString().padLeft(2, '0');
    final startStr = '${twoDigits(start.hour)}:${twoDigits(start.minute)}';
    final endStr = '${twoDigits(end.hour)}:${twoDigits(end.minute)}';
    return '$startStr – $endStr';
  }

  @override
  Widget build(BuildContext context) {
    // Spot Info
    final spotName = spotInfo == null ? spotGroup.spot : spotInfo!.name;
    const double iconLeftOffset = -96;

    return Padding(
      padding: const EdgeInsets.only(bottom: 44),
      child: SizedBox(
        width: 280,

        child: Column(
          mainAxisAlignment: MainAxisAlignment.start,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // SPOT
            Row(
              mainAxisSize: MainAxisSize.min,
              // mainAxisAlignment: MainAxisAlignment.center,
              children: [
                RegionalIcon(surfspot: spotInfo),
                // SPOT NAME
                Text(spotName, style: NwTextStyles.title.white),
              ],
            ),

            // All event time windows for this spot on this date (soonest first)
            Wrap(
              direction: Axis.vertical,
              spacing: 8,
              runSpacing: 8,
              children: spotGroup.events.map((event) {
                final Surfability surfability = event.label;
                // TIME & Surfability
                return Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    SizedBox(width: 55),

                    Column(
                      children: [
                        SurfabilityIcon(
                          surfability: event.label,
                          hourlyScores: event.hourlyScores,
                        ),

                        const SizedBox(height: 4),

                        // TIME RANGE
                        Container(
                          padding: const EdgeInsets.symmetric(
                            horizontal: 10,
                            vertical: 6,
                          ),
                          decoration: BoxDecoration(
                            // color: kNwPeri.withValues(alpha: 0.12),
                            color: surfability.color.withValues(alpha: 0.12),
                            borderRadius: BorderRadius.circular(8),
                            border: Border.all(
                              // color: kNwPeri.withValues(alpha: 0.45),
                              color: surfability.color.withValues(alpha: 0.45),
                            ),
                          ),
                          child: Text(
                            _formatTimeWindow(event.start, event.end),
                            // style: NwTextStyles.bodySmall.c(kNwPeri),
                            style: NwTextStyles.bodySmall.c(surfability.color),
                          ),
                        ),
                      ],
                    ),
                  ],
                );
              }).toList(),
            ),
          ],
        ),
      ),
    );
  }
}

class SurfabilityIcon extends StatelessWidget {
  const new({
    super.key,
    required this.surfability,
    this.hourlyScores,
  });

  final Surfability surfability;
  final Map<DateTime, double>? hourlyScores;

  @override
  Widget build(BuildContext context) {
    final name = surfability.name;
    final color = surfability.color;

    return Container(
      child: Row(
        // mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.end,
        // textBaseline: TextBaseline.ideographic,
        children: [
          // Surfability Label
          Text(name, style: NwTextStyles.bodyMedium.c(color).bold()),

          SizedBox(width: 4),

          //// signal
          // if (surfability == Surfability.surfschool)
          //   Icon(Icons.signal_cellular_alt_1_bar_rounded, color: color),

          // if (surfability == Surfability.surfable)
          //   Icon(Icons.signal_cellular_alt_2_bar_rounded, color: color),

          // if (surfability == Surfability.works)
          //   Icon(Icons.signal_cellular_alt_rounded, color: color),

          // if (surfability == Surfability.on)
          //   Icon(Icons.signal_cellular_alt_sharp, color: color),

          // BAR
          HourlyScoreBarChart(hourlyScores: hourlyScores),
        ],
      ),
    );
  }
}
