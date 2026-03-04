import 'package:arbiter/features/adaptive_switcher.dart';
import 'package:arbiter/screens/about.dart';
import 'package:arbiter/screens/calc.dart';
import 'package:flutter/material.dart';

final _destinations = [
  Destination(
    label: "About",
    icon: Icon(Icons.info_outline),
    main: AdaptiveBuilders(build: (_) => About()),
  ),
  Destination(
    label: "Calc",
    icon: Icon(Icons.calculate),
    main: AdaptiveBuilders(build: (_) => CalcScreen()),
  ),
];

class Home extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return HomeRouter(destinations: _destinations);
  }
}
