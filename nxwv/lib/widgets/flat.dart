import 'package:flutter/material.dart';
import 'package:nxwv/constants.dart';

// import 'package:flutter/math.dart' as math; // Optional if using radians explicitly, or just use doubles

class FlatTruck extends StatefulWidget {
  const FlatTruck({super.key});

  @override
  State<FlatTruck> createState() => _FlatTruckState();
}

class _FlatTruckState extends State<FlatTruck>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller;
  late final Animation<double> _animation;

  @override
  void initState() {
    super.initState();
    // 1. Setup the animation controller for a continuous rocking motion
    _controller = AnimationController(
      duration: const Duration(milliseconds: 400), // Speed of the rock
      vsync: this,
    )..repeat(reverse: true); // Go back and forth indefinitely

    // 2. Define the tilt angle range (in radians, e.g., roughly -3 to +3 degrees)
    _animation = Tween<double>(begin: -0.02, end: 0.03).animate(
      CurvedAnimation(
        parent: _controller,
        curve:
            Curves.easeInOut, // Smooth acceleration/deceleration at the edges
      ),
    );
  }

  @override
  void dispose() {
    _controller.dispose(); // Always dispose controllers to prevent memory leaks
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // 3. Wrap the icon in an AnimatedBuilder and Transform.rotate
        AnimatedBuilder(
          animation: _animation,
          builder: (context, child) {
            return Transform.rotate(angle: _animation.value, child: child);
          },
          child: Icon(Icons.fire_truck, size: 200, color: kNwMint),
        ),
        const SizedBox(height: 8),
        Text(
          'FLAT',
          style: NwTextStyles.title.c(kNwFlatRed).copyWith(fontSize: 50),
        ),

        Text('There are no incoming swells to report at this time'),
      ],
    );
  }
}
