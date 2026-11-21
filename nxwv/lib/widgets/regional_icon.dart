import 'package:flutter/material.dart';
import 'package:nxwv/models/spot_model.dart';

class RegionalIcon extends StatelessWidget {
  const new({super.key, this.surfspot});

  final SurfSpot? surfspot;

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
      child: CircleAvatar(child: Icon(iconData, size: 20)),
    );
  }
}
