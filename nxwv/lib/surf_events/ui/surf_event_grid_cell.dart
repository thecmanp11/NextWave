import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nxwv/constants.dart';
import 'package:nxwv/models/spot_model.dart';
import 'package:nxwv/models/surf_event_model.dart';
import 'package:nxwv/surf_events/ui/surf_event_groups.dart';
import 'package:nxwv/widgets/regional_icon.dart';

class SurfEventGridCell extends ConsumerWidget {
  const new({
    super.key,
    required this.surfEvent,
    this.showDate = true,
  });

  final SurfEvent surfEvent;
  final bool showDate;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final startDate = surfEvent.start;
    final endDate = surfEvent.end;
    final spot = surfEvent.spot;
    final label = surfEvent.label;
    final hourlyScores = surfEvent.hourlyScores;

    // joins
    final spotsMap = ref.watch(surfSpotProvider).value ?? const {};
    final spotData = spotsMap[spot];
    // TODO: sexify
    final spotName = spotData == null ? spot : spotData.name;

    return Container(
      height: 250,
      width: 250,
      child: Column(
        children: [
          // Level 1: Date Header
          Container(
            height: 66,
            alignment: const Alignment(-0.20, 0.0), // X% left of center
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: showDate
                ? Text(
                    _formatDateHeader(startDate),
                    style: NwTextStyles.title,
                    textAlign: TextAlign.left,
                    maxLines: 1,
                    softWrap: false,
                    overflow: TextOverflow.ellipsis,
                  )
                : null,
          ),

          // 2. SPOT
          Row(
            mainAxisSize: MainAxisSize.min,
            // mainAxisAlignment: MainAxisAlignment.center,
            children: [
              RegionalIcon(surfspot: spotData),
              // SPOT NAME
              Text(spotName, style: NwTextStyles.title.white),
            ],
          ),

          // 3. Surfability
          SurfabilityIcon(
            surfability: label,
            hourlyScores: hourlyScores,
          ),

          // 4. time
          Container(
            padding: const EdgeInsets.symmetric(
              horizontal: 10,
              vertical: 6,
            ),
            decoration: BoxDecoration(
              // color: kNwPeri.withValues(alpha: 0.12),
              color: label.color.withValues(alpha: 0.12),
              borderRadius: BorderRadius.circular(8),
              border: Border.all(
                // color: kNwPeri.withValues(alpha: 0.45),
                color: label.color.withValues(alpha: 0.45),
              ),
            ),
            child: Text(
              _formatTimeWindow(startDate, endDate),
              // style: NwTextStyles.bodySmall.c(kNwPeri),
              style: NwTextStyles.bodySmall.c(label.color),
            ),
          ),
        ],
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

  // TODO: don't want to localize time to MY location... I want the times for the SPOT
  String _formatTimeWindow(DateTime startUtc, DateTime endUtc) {
    final start = startUtc.toLocal();
    final end = endUtc.toLocal();
    String twoDigits(int n) => n.toString().padLeft(2, '0');
    final startStr = '${twoDigits(start.hour)}:${twoDigits(start.minute)}';
    final endStr = '${twoDigits(end.hour)}:${twoDigits(end.minute)}';
    return '$startStr – $endStr';
  }
}
