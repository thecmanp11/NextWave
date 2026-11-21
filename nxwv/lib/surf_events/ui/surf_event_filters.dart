import 'package:flutter/material.dart';

class SurfEventFilters extends StatelessWidget {
  const new({super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      height: 20,
      child: Container(
        width: 100,
        child: Row(children: [Text('Country'), Text('Regions')]),
      ),
    );
  }
}
