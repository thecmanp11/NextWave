import 'package:flutter/material.dart';
import 'package:nxwv/surf_events/scrolls/surf_event_grid.dart';
import 'package:nxwv/surf_events/scrolls/surf_event_scroll.dart';
import 'package:nxwv/surf_events/ui/surf_event_filters.dart';

class HomeBase extends StatelessWidget {
  const new({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('🧿 Next Wave: Mediterranean Surf Finder')),

      body: Column(
        children: [
          // SurfEventFilters(),
          // SurfEventScroll(),
          SurfEventGrid(),
        ],
      ),
    );
  }
}
