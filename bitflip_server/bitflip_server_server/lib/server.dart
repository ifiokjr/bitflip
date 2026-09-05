import 'dart:io';

import 'package:bitflip_server_server/src/generated/serverpod.dart';
import 'package:bitflip_server_server/src/minting/bitflip_mint_service.dart';
import 'package:bitflip_server_server/src/minting/section_routes.dart';
import 'package:bitflip_server_server/src/web/security_headers_middleware.dart';

/// The starting point of the Serverpod server.
void run(List<String> args) async {
  final pod = Serverpod(args);
  final mintService = SolanaBitflipMintService.fromEnvironment(
    Platform.environment,
    production: pod.runMode == ServerpodRunMode.production,
  );
  MintServiceRegistry.configure(mintService);
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
}
