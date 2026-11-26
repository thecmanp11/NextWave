import 'package:flutter/material.dart';
import 'package:nxwv/constants.dart';
import 'package:nxwv/models/surf_event_model.dart';
import 'package:nxwv/widgets/hourly_score_bar_chart.dart';

/*

Illustrates how surfable the spot/day is:
Surfability label
plus a SurfQuality chart for the spot/day by hour

*/

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
      // alignment: const Alignment(0.2, 0.0),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        mainAxisAlignment: MainAxisAlignment.center,
        // crossAxisAlignment: CrossAxisAlignment.end,
        // textBaseline: TextBaseline.ideographic,
        children: [
          // Surfability Label
          Text(name, style: NwTextStyles.bodyMedium.c(color).bold()),

          // BAR
          HourlyScoreBarChart(hourlyScores: hourlyScores),
        ],
      ),
    );
  }
}
