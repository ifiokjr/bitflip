import 'dart:convert';
import 'dart:math';

import 'package:bitflip_server_server/src/generated/protocol.dart';
import 'package:bitflip_server_server/src/minting/bitflip_mint_service.dart';
import 'package:bitflip_server_server/src/minting/solana_signature.dart';
import 'package:serverpod/serverpod.dart';
import 'package:solana_kit/solana_kit.dart';

final _secureRandom = Random.secure();
final _noncePattern = RegExp(r'^[A-Za-z0-9_-]{32}$');

class MintEndpoint extends Endpoint {
  static const _challengeLifetime = Duration(minutes: 5);
  static const _challengeRetention = Duration(days: 1);
  static const _chainReadDeadline = Duration(seconds: 10);
  static const _mintDeadline = Duration(seconds: 90);

  static final ChallengeRateLimiter _challengeLimiter = ChallengeRateLimiter();
  static final MintOperatorGate _operatorGate = MintOperatorGate();

  @override
  bool get requireLogin => false;

  Future<MintChallengeView> createChallenge(
    Session session, {
    required String walletAddress,
    required int gameIndex,
    required int sectionIndex,
  }) async {
    final wallet = _validatedWallet(walletAddress);
    _validateIndices(gameIndex, sectionIndex);
    _challengeLimiter.record(_requestSource(session));
    final chainSection = await MintServiceRegistry.service
        .loadSection(gameIndex, sectionIndex)
        .timeout(_chainReadDeadline);
    if (chainSection.owner != wallet) {
      throw StateError('Only the on-chain section owner can request a mint.');
    }
    if (!chainSection.isSealed && !chainSection.isMinted) {
      throw StateError('Seal the section before requesting a mint.');
    }

    final now = DateTime.now().toUtc();
    final expiresAt = now.add(_challengeLifetime);
    final nonce = _nonce();
    final message = mintAuthorizationMessage(
      walletAddress: wallet.value,
      gameIndex: gameIndex,
      sectionIndex: sectionIndex,
      nonce: nonce,
      expiresAt: expiresAt,
    );
    await DatabaseUtil.runInTransactionOrSavepoint(session.db, null, (
      transaction,
    ) async {
      await session.db.unsafeQuery(
        'SELECT pg_advisory_xact_lock(hashtextextended(@wallet, 0));',
        transaction: transaction,
        parameters: QueryParameters.named({'wallet': wallet.value}),
      );
      await MintChallenge.db.deleteWhere(
        session,
        where: (table) => table.expiresAt < now.subtract(_challengeRetention),
        transaction: transaction,
        noReturn: true,
      );
      await MintChallenge.db.insertRow(
        session,
        MintChallenge(
          walletAddress: wallet.value,
          gameIndex: gameIndex,
          sectionIndex: sectionIndex,
          nonce: nonce,
          message: message,
          createdAt: now,
          expiresAt: expiresAt,
        ),
        transaction: transaction,
      );
    });
    return MintChallengeView(
      nonce: nonce,
      message: message,
      expiresAt: expiresAt,
    );
  }

  Future<MintSectionResult> mintSection(
    Session session, {
    required String walletAddress,
    required int gameIndex,
    required int sectionIndex,
    required String nonce,
    required String signatureBase64,
  }) async {
    final wallet = _validatedWallet(walletAddress);
    _validateIndices(gameIndex, sectionIndex);
    final validatedNonce = _validatedNonce(nonce);
    final startedAt = DateTime.now().toUtc();
    try {
      final result = await _operatorGate.run(() async {
        await DatabaseUtil.runInTransactionOrSavepoint(session.db, null, (
          transaction,
        ) async {
          final challenge = await MintChallenge.db.findFirstRow(
            session,
            where: (table) =>
                table.walletAddress.equals(wallet.value) &
                table.gameIndex.equals(gameIndex) &
                table.sectionIndex.equals(sectionIndex) &
                table.nonce.equals(validatedNonce),
            transaction: transaction,
            lockMode: LockMode.forUpdate,
          );
          final now = DateTime.now().toUtc();
          if (challenge == null ||
              challenge.usedAt != null ||
              !challenge.expiresAt.isAfter(now)) {
            throw StateError('The mint authorization is no longer valid.');
          }
          final expectedMessage = mintAuthorizationMessage(
            walletAddress: wallet.value,
            gameIndex: gameIndex,
            sectionIndex: sectionIndex,
            nonce: challenge.nonce,
            expiresAt: challenge.expiresAt,
          );
          if (challenge.message != expectedMessage ||
              !verifySolanaSignature(
                publicKeyBytes: decodeBase58PublicKey(wallet.value),
                message: expectedMessage,
                signatureBytes: decodeBase64Signature(signatureBase64),
              )) {
            throw StateError('The mint authorization signature is invalid.');
          }

          await MintChallenge.db.updateRow(
            session,
            challenge.copyWith(usedAt: now),
            transaction: transaction,
          );
        });
        final section = await MintServiceRegistry.service
            .loadSection(gameIndex, sectionIndex)
            .timeout(_chainReadDeadline);
        if (section.owner != wallet) {
          throw StateError('The section owner changed before minting.');
        }
        return MintServiceRegistry.service.mint(section).timeout(_mintDeadline);
      });
      _logMint(
        session,
        gameIndex: gameIndex,
        sectionIndex: sectionIndex,
        startedAt: startedAt,
        outcome: result.alreadyMinted ? 'already_minted' : 'confirmed',
      );
      return MintSectionResult(
        assetId: result.assetId.value,
        merkleTree: result.merkleTree.value,
        leafIndex: result.leafIndex,
        transactionSignature: result.transactionSignature,
        alreadyMinted: result.alreadyMinted,
      );
    } on Object catch (error, stackTrace) {
      _logMint(
        session,
        gameIndex: gameIndex,
        sectionIndex: sectionIndex,
        startedAt: startedAt,
        outcome: 'failed',
        error: error,
        stackTrace: stackTrace,
      );
      rethrow;
    }
  }
}

/// Process-local protection for the public challenge endpoint. Production
/// deployments must pair this with a distributed edge rate limit.
final class ChallengeRateLimiter {
  ChallengeRateLimiter({
    this.window = const Duration(minutes: 1),
    this.maximumPerSource = 30,
    this.maximumGlobal = 240,
    DateTime Function()? clock,
  }) : _clock = clock ?? DateTime.now;

  final Duration window;
  final int maximumPerSource;
  final int maximumGlobal;
  final DateTime Function() _clock;
  final Map<String, List<DateTime>> _attemptsBySource = {};
  final List<DateTime> _globalAttempts = [];

  void record(String source) {
    final now = _clock().toUtc();
    final cutoff = now.subtract(window);
    _globalAttempts.removeWhere((attempt) => attempt.isBefore(cutoff));
    if (_globalAttempts.isEmpty) {
      _attemptsBySource.removeWhere(
        (_, attempts) => attempts.every((attempt) => attempt.isBefore(cutoff)),
      );
    }
    final attempts = _attemptsBySource.putIfAbsent(source, () => []);
    attempts.removeWhere((attempt) => attempt.isBefore(cutoff));
    if (attempts.length >= maximumPerSource ||
        _globalAttempts.length >= maximumGlobal) {
      throw StateError('Too many mint requests. Wait before trying again.');
    }
    attempts.add(now);
    _globalAttempts.add(now);
  }
}

/// Rejects excess operator work immediately instead of tying up server and
/// database resources while Solana RPC calls are in flight.
final class MintOperatorGate {
  MintOperatorGate({this.maximumInFlight = 1});

  final int maximumInFlight;
  int _inFlight = 0;

  int get inFlight => _inFlight;

  Future<T> run<T>(Future<T> Function() action) async {
    if (_inFlight >= maximumInFlight) {
      throw StateError('The mint operator is busy. Try again shortly.');
    }
    _inFlight++;
    try {
      return await action();
    } finally {
      _inFlight--;
    }
  }
}

String _requestSource(Session session) {
  return session.request?.connectionInfo.remote.address.toString() ??
      'internal';
}

void _logMint(
  Session session, {
  required int gameIndex,
  required int sectionIndex,
  required DateTime startedAt,
  required String outcome,
  Object? error,
  StackTrace? stackTrace,
}) {
  session.log(
    'Bitflip mint completed.',
    level: error == null ? LogLevel.info : LogLevel.error,
    exception: error,
    stackTrace: stackTrace,
    metadata: {
      'operation': 'mint_section',
      'gameIndex': gameIndex,
      'sectionIndex': sectionIndex,
      'outcome': outcome,
      'durationMs': DateTime.now().toUtc().difference(startedAt).inMilliseconds,
    },
  );
}

String mintAuthorizationMessage({
  required String walletAddress,
  required int gameIndex,
  required int sectionIndex,
  required String nonce,
  required DateTime expiresAt,
}) {
  return [
    'Bitflip compressed NFT authorization',
    'Wallet: $walletAddress',
    'Game: $gameIndex',
    'Section: $sectionIndex',
    'Nonce: $nonce',
    'Expires: ${expiresAt.toUtc().toIso8601String()}',
    'Action: Mint the sealed section to this wallet.',
  ].join('\n');
}

Address _validatedWallet(String value) {
  final normalized = value.trim();
  if (normalized.length < 32 || normalized.length > 44) {
    throw const FormatException('Invalid Solana wallet address.');
  }
  final wallet = Address(normalized);
  decodeBase58PublicKey(wallet.value);
  return wallet;
}

String _validatedNonce(String value) {
  final normalized = value.trim();
  if (!_noncePattern.hasMatch(normalized)) {
    throw const FormatException('Invalid mint authorization nonce.');
  }
  return normalized;
}

void _validateIndices(int gameIndex, int sectionIndex) {
  if (gameIndex < 0 || gameIndex > 255) {
    throw RangeError.range(gameIndex, 0, 255, 'gameIndex');
  }
  if (sectionIndex < 0 || sectionIndex > 255) {
    throw RangeError.range(sectionIndex, 0, 255, 'sectionIndex');
  }
}

String _nonce() {
  final bytes = List<int>.generate(24, (_) => _secureRandom.nextInt(256));
  return base64UrlEncode(bytes).replaceAll('=', '');
}
