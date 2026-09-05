import 'package:bitflip_app/app/app.dart';
import 'package:bitflip_app/core/bitflip_config.dart';
import 'package:flutter/material.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  final config = BitflipConfig.fromEnvironment();
  runApp(
    ProviderScope(
      overrides: [bitflipConfigProvider.overrideWithValue(config)],
      child: const BitflipApp(),
    ),
  );
}
