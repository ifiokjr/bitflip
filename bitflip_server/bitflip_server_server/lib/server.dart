import 'dart:io';

import 'package:bitflip_program/bitflip_program.dart';
import 'package:bitflip_server_server/src/colour/colour_event_indexer.dart';
import 'package:bitflip_server_server/src/colour/colour_flip_event_source.dart';
import 'package:bitflip_server_server/src/generated/serverpod.dart';
import 'package:bitflip_server_server/src/minting/bitflip_mint_service.dart';
import 'package:bitflip_server_server/src/minting/section_routes.dart';
import 'package:bitflip_server_server/src/web/security_headers_middleware.dart';
import 'package:bitflip_server_server/src/web/solana_rpc_health_indicator.dart';

/// The starting point of the Serverpod server.
void run(List<String> args) async {
  final runMode =
      _argumentValue(args, '--mode') ??
      Platform.environment['runmode'] ??
      ServerpodRunMode.development;
  final mintService = SolanaBitflipMintService.fromEnvironment(
    Platform.environment,
    production: runMode == ServerpodRunMode.production,
    requireExplicitConfiguration: runMode == ServerpodRunMode.staging,
  );
  final strictConfiguration =
      runMode == ServerpodRunMode.production ||
      runMode == ServerpodRunMode.staging;
  final colourIndexerConfiguration = ColourIndexerConfiguration.fromEnvironment(
    Platform.environment,
    requireEnabled: strictConfiguration,
  );
  final pod = Serverpod(
    args,
    healthConfig: HealthConfig(
      cacheTtl: const Duration(seconds: 5),
      additionalReadinessIndicators: [
        SolanaRpcHealthIndicator.forService(mintService),
      ],
    ),
  );
  MintServiceRegistry.configure(mintService);
  final colourEventSource = SolanaColourFlipEventSource(mintService.rpc);
  ColourFlipEventSourceRegistry.configure(colourEventSource);
  ColourEventIndexerRegistry.configure(
    ColourEventIndexer(
      configuration: colourIndexerConfiguration,
      signatureSource: colourEventSource,
      eventSource: colourEventSource,
    ),
  );
  pod.webServer.addMiddleware(const SecurityHeadersMiddleware().call, '/');

  pod.webServer
    ..addRoute(
      SectionMetadataRoute(
        mintService: mintService,
        publicBaseUrl: mintService.metadataBaseUrl,
      ),
      '/metadata/:game/:section',
    )
    ..addRoute(SectionArtRoute(mintService), '/art/:game/:section');

  final flutterWeb = Directory('web/app');
  if (flutterWeb.existsSync()) {
    pod.webServer.addRoute(FlutterRoute(flutterWeb));
  }

  await pod.start();
  if (colourIndexerConfiguration.enabled) {
    final identifier =
        'colour-indexer-${colourIndexerConfiguration.cluster}-'
        '${bitflipProgramProgramAddress.value}';
    await pod.futureCalls.cancel(identifier);
    await pod.futureCalls
        .callRecurring(identifier: identifier)
        .every(
          colourIndexerConfiguration.interval,
          start: DateTime.now().toUtc(),
        )
        .colourIndexer
        .scan();
  }
}

String? _argumentValue(List<String> args, String name) {
  for (var index = 0; index < args.length; index++) {
    final argument = args[index];
    if (argument == name && index + 1 < args.length) return args[index + 1];
    if (argument.startsWith('$name=')) {
      return argument.substring(name.length + 1);
    }
  }
  return null;
}
