// import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:nxwv/constants.dart';

import 'dart:math' as math;

import 'package:nxwv/models/surf_event_model.dart';

class HourlyScoreBarChart extends StatelessWidget {
  const HourlyScoreBarChart({
    super.key,
    required this.hourlyScores,
    this.startHour = 5,
    this.endHour = 21,
    this.width = 50,
    this.height = 20,
    this.emptyColor = const Color(0x14808080),
    this.useLocalTime = true,
    this.showHourLabels = false,
  }) : assert(endHour >= startHour);

  final Map<DateTime, double>? hourlyScores;
  final int startHour;
  final int endHour;
  final double width;
  final double height;
  final Color emptyColor;
  final bool useLocalTime;
  final bool showHourLabels;

  @override
  Widget build(BuildContext context) {
    // final theme = Theme.of(context);

    final scoresByHour = <int, double>{};
    if (hourlyScores != null) {
      for (final entry in hourlyScores!.entries) {
        final dt = useLocalTime ? entry.key.toLocal() : entry.key.toUtc();
        if (dt.hour >= startHour && dt.hour <= endHour) {
          scoresByHour[dt.hour] = entry.value;
        }
      }
    }

    return SizedBox(
      width: width,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          SizedBox(
            width: width,
            height: height,
            child: CustomPaint(
              size: Size(width, height),
              painter: _HourlyBarPainter(
                scoresByHour: scoresByHour,
                startHour: startHour,
                endHour: endHour,
                emptyColor: emptyColor,
              ),
            ),
          ),
          // if (showHourLabels) ...[
          //   const SizedBox(height: 2),
          //   Row(
          //     mainAxisAlignment: MainAxisAlignment.spaceBetween,
          //     children: [
          //       Text(
          //         '$startHour',
          //         style: theme.textTheme.labelSmall?.copyWith(
          //           fontSize: 9,
          //           color: theme.colorScheme.onSurfaceVariant,
          //         ),
          //       ),
          //       Text(
          //         '${((startHour + endHour) ~/ 2)}',
          //         style: theme.textTheme.labelSmall?.copyWith(
          //           fontSize: 9,
          //           color: theme.colorScheme.onSurfaceVariant,
          //         ),
          //       ),
          //       Text(
          //         '$endHour',
          //         style: theme.textTheme.labelSmall?.copyWith(
          //           fontSize: 9,
          //           color: theme.colorScheme.onSurfaceVariant,
          //         ),
          //       ),
          //     ],
          //   ),
          // ],
        ],
      ),
    );
  }
}

class _HourlyBarPainter extends CustomPainter {
  _HourlyBarPainter({
    required this.scoresByHour,
    required this.startHour,
    required this.endHour,
    required this.emptyColor,
  });

  final Map<int, double> scoresByHour;
  final int startHour;
  final int endHour;
  final Color emptyColor;

  @override
  void paint(Canvas canvas, Size size) {
    final slotCount = endHour - startHour + 1;
    if (slotCount <= 0 || size.width <= 0 || size.height <= 0) return;

    const gap = 1.5;
    final slotWidth = (size.width - gap * (slotCount - 1)) / slotCount;
    final barWidth = math.max(1.5, slotWidth);

    final hasData = scoresByHour.isNotEmpty;
    final minScore = hasData
        ? scoresByHour.values.reduce((a, b) => a < b ? a : b)
        : 0.0;
    final maxScore = hasData
        ? scoresByHour.values.reduce((a, b) => a > b ? a : b)
        : 100.0;
    final span = maxScore - minScore;

    final trackPaint = Paint()..color = emptyColor;
    final barPaint = Paint();
    final radius = Radius.circular(math.min(2.0, barWidth / 2));

    for (var i = 0; i < slotCount; i++) {
      final hour = startHour + i;
      final x = i * (slotWidth + gap);

      canvas.drawRRect(
        RRect.fromRectAndRadius(
          Rect.fromLTWH(x, 0, barWidth, size.height),
          radius,
        ),
        trackPaint,
      );

      final score = scoresByHour[hour];
      if (score != null) {
        final normalized = span > 0 ? (score - minScore) / span : 1.0;
        final ratio = 0.2 + 0.8 * normalized.clamp(0.0, 1.0);
        final barHeight = math.max(2.0, size.height * ratio);
        final y = size.height - barHeight;

        barPaint.color = Surfability.fromPeakScore(score).color;

        canvas.drawRRect(
          RRect.fromRectAndRadius(
            Rect.fromLTWH(x, y, barWidth, barHeight),
            radius,
          ),
          barPaint,
        );
      }
    }
  }

  @override
  bool shouldRepaint(covariant _HourlyBarPainter oldDelegate) {
    return oldDelegate.scoresByHour != scoresByHour ||
        oldDelegate.startHour != startHour ||
        oldDelegate.endHour != endHour ||
        oldDelegate.emptyColor != emptyColor;
  }
}
