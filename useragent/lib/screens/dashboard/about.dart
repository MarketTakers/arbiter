import 'package:auto_route/auto_route.dart';
import 'package:flutter/material.dart';
import 'package:mtcore/markettakers.dart' as mt;

@RoutePage()
class AboutScreen extends StatelessWidget {
  const AboutScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return mt.AboutScreen(decription: "Arbiter is bla bla bla");
  }
}
