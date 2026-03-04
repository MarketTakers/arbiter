import 'package:arbiter/router.dart';
import 'package:flutter/material.dart' hide Router;
import 'package:hooks_riverpod/hooks_riverpod.dart';

void main() {
  runApp(
    MaterialApp(
      home: ProviderScope(child: Scaffold(body: Router())),
    ),
  );
}
