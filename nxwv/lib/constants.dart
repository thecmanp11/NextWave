import 'package:flutter/material.dart';

const kNwMint = Color.fromARGB(255, 75, 255, 205);

const kNwLBlue = Color.fromARGB(255, 53, 230, 255);

const kNwNavy = Color.fromARGB(255, 0, 16, 39);
final Color kNwBkg = HSLColor.fromColor(kNwNavy)
    .withLightness(
      (HSLColor.fromColor(kNwNavy).lightness - 0.04).clamp(0.0, 1.0),
    )
    .toColor();

const kNwPeri = Color.fromARGB(255, 235, 180, 255);

const kNwFlatRed = Colors.red;

const kCutePink = Color.fromARGB(255, 255, 175, 175);

const kUnselectedGrey = Color.fromRGBO(66, 66, 66, 1);

const kLightGrey = Color.fromRGBO(96, 96, 96, 1);

const kDarkBlack = Colors.black;

/// Base color-agnostic text styles (Fonts, sizes, weights, heights)
class NwTextStyles {
  // Headings & Titles
  static const TextStyle display = TextStyle(
    fontSize: 32,
    fontWeight: FontWeight.bold,
    height: 1.2,
  );

  static const TextStyle title = TextStyle(
    fontSize: 22,
    fontWeight: FontWeight.bold,
    height: 1.3,
  );

  static const TextStyle subtitle = TextStyle(
    fontSize: 16,
    fontWeight: FontWeight.w600,
    height: 1.4,
  );

  // Standard Body Text Sizes
  static const TextStyle bodyLarge = TextStyle(
    fontSize: 16,
    fontWeight: FontWeight.normal,
    height: 1.5,
  );

  static const TextStyle bodyMedium = TextStyle(
    fontSize: 14,
    fontWeight: FontWeight.normal,
    height: 1.5,
  );

  static const TextStyle bodySmall = TextStyle(
    fontSize: 12,
    fontWeight: FontWeight.normal,
    height: 1.4,
  );

  // Special / Monospace (Great for your Python/Rust logs or terminal outputs)
  static const TextStyle code = TextStyle(
    fontFamily: 'monospace',
    fontSize: 13,
    fontWeight: FontWeight.w500,
  );
}

/// Extension to easily map ANY color to your text styles
extension TextStyleColorExtension on TextStyle {
  /// Maps any color to the text style
  TextStyle c(Color color) => copyWith(color: color);

  /// Quick shorthand for white (since it's your primary default text)
  TextStyle get white => copyWith(color: Colors.white);
}

// --- App Theme Setup ---
final ThemeData kNwDarkTheme = ThemeData(
  useMaterial3: true,
  brightness: Brightness.dark,
  scaffoldBackgroundColor: kNwBkg,
  canvasColor: kNwNavy,

  // Customizing the Material 3 ColorScheme based on your palette
  colorScheme: const ColorScheme.dark(
    primary: kNwLBlue, // Main interactive elements
    onPrimary: kNwNavy, // Text/icons on top of primary
    secondary: kNwMint, // Accent color
    onSecondary: kNwNavy, // Text/icons on top of secondary
    tertiary: kNwPeri, // Tertiary accent (Periwinkle)
    onTertiary: kNwNavy,
    surface: kNwNavy, // Cards, sheets, backgrounds
    onSurface: Colors.white, // Main text color across surfaces
    error: kNwFlatRed,
    onError: Colors.white,
  ),

  // Global Text Theme settings ensuring main text defaults to white
  textTheme: const TextTheme(
    displayLarge: TextStyle(color: Colors.white),
    displayMedium: TextStyle(color: Colors.white),
    displaySmall: TextStyle(color: Colors.white),
    headlineLarge: TextStyle(color: Colors.white),
    headlineMedium: TextStyle(color: Colors.white),
    headlineSmall: TextStyle(color: Colors.white),
    titleLarge: TextStyle(color: Colors.white, fontWeight: FontWeight.bold),
    titleMedium: TextStyle(color: Colors.white),
    titleSmall: TextStyle(color: Colors.white70),
    bodyLarge: TextStyle(color: Colors.white),
    bodyMedium: TextStyle(color: Colors.white70),
    bodySmall: TextStyle(color: Colors.white60),
  ),

  // Global App Bar styling to match the deep navy background
  appBarTheme: const AppBarTheme(
    backgroundColor: kNwNavy,
    foregroundColor: Colors.white,
    elevation: 0,
    centerTitle: true,
  ),
);
