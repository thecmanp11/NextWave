import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:nxwv/constants.dart';
import 'package:nxwv/surf_events/scrolls/surf_event_grid.dart';
import 'package:nxwv/surf_events/ui/surf_event_filters.dart';

class HomeBase extends StatelessWidget {
  const new({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      extendBodyBehindAppBar: true,
      appBar: AppBar(
        automaticallyImplyLeading: false,
        elevation: 0,
        scrolledUnderElevation: 0,
        backgroundColor: Colors.transparent,
        surfaceTintColor: Colors.transparent,
        flexibleSpace: IgnorePointer(
          child: DecoratedBox(
            decoration: BoxDecoration(
              gradient: LinearGradient(
                begin: Alignment.topCenter,
                end: Alignment.bottomCenter,
                colors: [kNwBkg, kNwBkg.withValues(alpha: 0.0)],
              ),
            ),
          ),
        ),

        centerTitle: true,
        title: Text(
          'NX🧿WV',
          style: NwTextStyles.title.c(kNwMint).bold().copyWith(fontSize: 44),
        ),
      ),
      body: Column(
        children: [
          SizedBox(height: 44),
          // SurfEventFilters(),
          SurfEventGrid(),
        ],
      ),
    );
  }
}
