import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nxwv/constants.dart';
import 'package:nxwv/models/spot_model.dart';
import 'package:nxwv/models/surf_event_model.dart';
import 'package:nxwv/widgets/surfability_icon.dart';
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
      height: 240,
      width: 240,
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          GlowingSurfCard(
            surfEvent: surfEvent,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // 2. SPOT
                Row(
                  mainAxisSize: MainAxisSize.min,
                  // mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    RegionalIcon(
                      surfspot: spotData,
                      color: label.color,
                    ),
                    // SPOT NAME
                    Text(spotName, style: NwTextStyles.title.white),
                  ],
                ),

                // 3. Surfability
                Center(
                  child: SurfabilityIcon(
                    surfability: label,
                    hourlyScores: hourlyScores,
                  ),
                ),

                // 4. time
                Align(
                  // alignment: const Alignment(0.2, 0.0),
                  alignment: AlignmentGeometry.center,
                  child: Container(
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
                ),
              ],
            ),
          ),
        ],
      ),
    );
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

class GlowingSurfCard extends StatelessWidget {
  const new({super.key, required this.surfEvent, required this.child});

  final SurfEvent surfEvent;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    final glowColor = surfEvent.label.color;
    return Container(
      padding: EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: kNwBkg, // Matches the app background inside the card
        borderRadius: BorderRadius.circular(10),
        border: Border.all(
          color: glowColor.withValues(alpha: 0.1),
          width: 1,
        ),
        boxShadow: [
          // Outer rim glow around the border
          BoxShadow(
            color: glowColor.withValues(alpha: 0.99),
            blurRadius: 14,
            spreadRadius: 0,
            offset: Offset.zero,
          ),
          // Subtle downward elevation shadow so it reads as floating above the scaffold
          BoxShadow(
            color: Colors.black.withValues(alpha: 0.1),
            blurRadius: 16,
            spreadRadius: -2,
            offset: const Offset(0, 8),
          ),
        ],
      ),

      child: child,
    );
  }
}
