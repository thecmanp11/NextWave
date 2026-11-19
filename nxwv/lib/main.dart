import 'package:flutter/material.dart';
import 'package:nxwv/constants.dart';
import 'package:nxwv/home.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  // This widget is the root of your application.
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Next Wave',
      theme: kNwDarkTheme,
      home: const HomeBase(),
    );
  }
}
