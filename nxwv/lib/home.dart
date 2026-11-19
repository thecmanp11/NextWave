import 'package:flutter/material.dart';
import 'package:nxwv/constants.dart';
import 'package:nxwv/widgets/flat.dart';

class HomeBase extends StatelessWidget {
  const new({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('🧿 Next Wave: Mediterranean Surf Finder')),

      body: Column(
        children: [
          Text('Surfability Meter', style: NwTextStyles.title),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceAround,
            children: [
              Text('surfschool', style: NwTextStyles.subtitle.c(kNwPeri)),
              Text('surfable', style: NwTextStyles.subtitle.c(kNwLBlue)),
              Text('working', style: NwTextStyles.subtitle.c(kNwMint)),
              Text('ON', style: NwTextStyles.subtitle.c(kNwMint)),
            ],
          ),

          FlatTruck(),
        ],
      ),
    );
  }
}
