import 'package:flutter/material.dart';
import 'package:nxwv/constants.dart';
import 'package:nxwv/models/spot_model.dart';

class RegionalIcon extends StatelessWidget {
  const new({
    super.key,
    this.surfspot,
    this.color = kNwLBlue,
  });

  final SurfSpot? surfspot;
  final Color color;

  @override
  Widget build(BuildContext context) {
    if (surfspot == null) {
      return _regionIcon(Icons.waves);
    } else {
      final String region = surfspot!.region;
      if (region == 'Peloponnese') {
        return _regionIcon(Icons.stacked_line_chart_rounded);
      } else {
        return _regionIcon(Icons.surfing);
      }
    }
  }

  Widget _regionIcon(IconData iconData) {
    return Padding(
      padding: const EdgeInsets.all(8.0),
      child: CircleAvatar(
        backgroundColor: color,
        child: Icon(iconData, size: 20),
      ),
    );
  }
}
