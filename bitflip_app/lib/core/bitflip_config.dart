import 'package:bitflip_program/bitflip_program_constraints.dart';
import 'package:flutter/foundation.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

enum BitflipEnvironment { development, staging, production }

final bitflipConfigProvider = Provider<BitflipConfig>(
  (ref) => BitflipConfig.fromEnvironment(),
);

final class BitflipConfig {
  const BitflipConfig({
    required this.environment,
    required this.walletChain,
    required this.rpcUrl,
    required this.serverpodUrl,
    required this.gameIndex,
    this.priorityFeeMicroLamports = 0,
  });

  factory BitflipConfig.fromEnvironment({bool releaseMode = kReleaseMode}) {
    const environment = String.fromEnvironment('BITFLIP_ENVIRONMENT');
    const walletChain = String.fromEnvironment('SOLANA_WALLET_CHAIN');
    const rpcUrl = String.fromEnvironment('SOLANA_RPC_URL');
    const serverpodUrl = String.fromEnvironment('SERVERPOD_URL');
    const gameIndex = String.fromEnvironment('BITFLIP_GAME_INDEX');
    const priorityFee = String.fromEnvironment(
      'SOLANA_PRIORITY_FEE_MICRO_LAMPORTS',
    );
    return BitflipConfig.parse(
      environment: environment,
      walletChain: walletChain,
      rpcUrl: rpcUrl,
      serverpodUrl: serverpodUrl,
      gameIndex: gameIndex,
      priorityFeeMicroLamports: priorityFee,
      releaseMode: releaseMode,
    );
  }

  factory BitflipConfig.parse({
    required String environment,
    required String walletChain,
    required String rpcUrl,
    required String serverpodUrl,
    required String gameIndex,
    required bool releaseMode,
    String priorityFeeMicroLamports = '',
  }) {
    if (!releaseMode) {
      environment = environment.isEmpty ? 'development' : environment;
      walletChain = walletChain.isEmpty ? 'solana:devnet' : walletChain;
      rpcUrl = rpcUrl.isEmpty ? 'https://api.devnet.solana.com' : rpcUrl;
      serverpodUrl = serverpodUrl.isEmpty
          ? 'http://localhost:8080/'
          : serverpodUrl;
      gameIndex = gameIndex.isEmpty ? '0' : gameIndex;
    }

    final missing = <String>[
      if (environment.trim().isEmpty) 'BITFLIP_ENVIRONMENT',
      if (walletChain.trim().isEmpty) 'SOLANA_WALLET_CHAIN',
      if (rpcUrl.trim().isEmpty) 'SOLANA_RPC_URL',
      if (serverpodUrl.trim().isEmpty) 'SERVERPOD_URL',
      if (gameIndex.trim().isEmpty) 'BITFLIP_GAME_INDEX',
    ];

    if (missing.isNotEmpty) {
      throw StateError(
        'Missing required Bitflip configuration: ${missing.join(', ')}.',
      );
    }

    final parsedEnvironment = switch (environment.trim()) {
      'development' => BitflipEnvironment.development,
      'staging' => BitflipEnvironment.staging,
      'production' => BitflipEnvironment.production,
      _ => throw StateError('BITFLIP_ENVIRONMENT is invalid.'),
    };
    if (!const {
      'solana:devnet',
      'solana:testnet',
      'solana:mainnet',
    }.contains(walletChain)) {
      throw StateError('SOLANA_WALLET_CHAIN is invalid.');
    }

    final parsedRpcUrl = _validatedUrl('SOLANA_RPC_URL', rpcUrl);
    final parsedServerpodUrl = _validatedUrl('SERVERPOD_URL', serverpodUrl);
    final parsedGameIndex = int.tryParse(gameIndex);
    if (parsedGameIndex == null ||
        parsedGameIndex < 0 ||
        parsedGameIndex > bitflipMaximumGameIndex) {
      throw StateError(
        'BITFLIP_GAME_INDEX must be between 0 and '
        '$bitflipMaximumGameIndex.',
      );
    }

    // An unset fee means no SetComputeUnitPrice instruction is attached.
    final parsedPriorityFee = priorityFeeMicroLamports.trim().isEmpty
        ? 0
        : int.tryParse(priorityFeeMicroLamports.trim());
    if (parsedPriorityFee == null || parsedPriorityFee < 0) {
      throw StateError(
        'SOLANA_PRIORITY_FEE_MICRO_LAMPORTS must be a non-negative integer.',
      );
    }

    if (parsedEnvironment == BitflipEnvironment.production) {
      if (walletChain != 'solana:mainnet') {
        throw StateError('Production must use solana:mainnet.');
      }

      _requirePublicHttps('SOLANA_RPC_URL', parsedRpcUrl);
      _requirePublicHttps('SERVERPOD_URL', parsedServerpodUrl);
    }

    return BitflipConfig(
      environment: parsedEnvironment,
      walletChain: walletChain,
      rpcUrl: parsedRpcUrl.toString(),
      serverpodUrl: parsedServerpodUrl.toString(),
      gameIndex: parsedGameIndex,
      priorityFeeMicroLamports: parsedPriorityFee,
    );
  }

  final BitflipEnvironment environment;
  final String walletChain;
  final String rpcUrl;
  final String serverpodUrl;
  final int gameIndex;

  /// Micro-lamports per compute unit added to every transaction as a priority
  /// fee; zero attaches no price and relies on the base fee alone.
  final int priorityFeeMicroLamports;
}

Uri _validatedUrl(String name, String value) {
  final uri = Uri.tryParse(value);

  if (uri == null || !uri.hasScheme || !uri.hasAuthority) {
    throw StateError('$name must be an absolute URL.');
  }

  if (uri.scheme != 'http' && uri.scheme != 'https') {
    throw StateError('$name must use HTTP or HTTPS.');
  }

  return uri;
}

void _requirePublicHttps(String name, Uri uri) {
  final host = uri.host.toLowerCase();
  if (uri.scheme != 'https' ||
      host == 'localhost' ||
      host == '127.0.0.1' ||
      host == '::1') {
    throw StateError('$name must use a public HTTPS endpoint in production.');
  }
}
