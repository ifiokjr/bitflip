import 'dart:convert';
import 'dart:typed_data';

import 'package:bitflip_app/core/bitflip_config.dart';
import 'package:bitflip_app/core/bitflip_wallet.dart';
import 'package:bitflip_app/features/game/data/bitflip_repository.dart';
import 'package:bitflip_app/features/game/domain/game_snapshot.dart';
import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';
import 'package:bitflip_app/features/game/domain/pixel_colour_map.dart';
import 'package:bitflip_app/features/game/domain/section_economy.dart';
import 'package:bitflip_app/features/game/domain/section_policy.dart';
import 'package:bitflip_program/bitflip_program.dart';
import 'package:bitflip_server_client/bitflip_server_client.dart' as serverpod;
import 'package:flutter_test/flutter_test.dart';
import 'package:solana_kit/solana_kit.dart';

final _walletAddress = _base58(32, 1);
final _treasuryAddress = _base58(32, 2);
final _bitMintAddress = _base58(32, 4);
final _bitReserveAddress = _base58(32, 5);
final _bitVaultAddress = _base58(32, 6);

void main() {
  late _ChainScript script;
  late _FakeWallet wallet;

  setUp(() {
    script = _ChainScript();
    wallet = _FakeWallet(_walletAddress);
  });

  SolanaBitflipRepository repository({
    bool connectWallet = true,
    Future<serverpod.ColourCanvasView?> Function(int, int, ColourCanvasHint?)?
    colourCanvasLoader,
  }) {
    return SolanaBitflipRepository(
      config: _config,
      rpc: createSolanaRpcFromTransport(script.transport),
      wallet: connectWallet ? wallet : _FakeWallet(null),
      colourCanvasLoader: colourCanvasLoader,
    );
  }

  test('builds its own RPC, address book, and wallet by default', () {
    final repository = SolanaBitflipRepository(config: _config);

    expect(repository.isDemoMode, isFalse);
    expect(repository.walletChain, 'solana:devnet');
  });

  group('SolanaBitflipRepository.loadRefresh', () {
    test(
      'reads the snapshot and wallet balance in one accounts fetch',
      () async {
        await script.serveGame();

        final refresh = await repository().loadRefresh(0);

        expect(refresh.snapshot?.section.revision, BigInt.from(7));
        expect(refresh.snapshot?.section.owner, _walletAddress);
        expect(refresh.snapshot?.section.lifecycle, SectionLifecycle.active);
        expect(refresh.snapshot?.claimPriceLamports, BigInt.from(1_000_000));
        expect(refresh.snapshot?.priceConfig?.windowSeconds, BigInt.from(300));
        expect(refresh.walletBalanceLamports, BigInt.from(1_234_567_890));
        expect(script.methods, unorderedEquals(['getMultipleAccounts']));

        final requested = script.requestedAddresses;

        expect(requested, hasLength(4));
        expect(requested.last, _walletAddress);
      },
    );

    test('skips the wallet read when no wallet is connected', () async {
      await script.serveGame();

      final refresh = await repository(connectWallet: false).loadRefresh(3);

      expect(refresh.snapshot?.section.lifecycle, SectionLifecycle.unclaimed);
      expect(refresh.walletBalanceLamports, isNull);

      final requested = script.requestedAddresses;

      expect(
        requested,
        hasLength(4),
        reason: 'config, game, section, previous',
      );
      expect(requested, isNot(contains(_walletAddress)));
      expect(requested[2], isNot(equals(requested[3])));
    });

    test('reports an unfunded wallet as a zero balance', () async {
      await script.serveGame(walletFunded: false);

      final refresh = await repository().loadRefresh(0);

      expect(refresh.snapshot, isNotNull);
      expect(refresh.walletBalanceLamports, BigInt.zero);
    });

    test('returns a null snapshot when the game is not initialized', () async {
      await script.serveGame(serveConfig: false, serveGameAccount: false);

      final refresh = await repository().loadRefresh(0);

      expect(refresh.snapshot, isNull);
      expect(refresh.walletBalanceLamports, BigInt.from(1_234_567_890));
    });

    test('rejects section indices outside the canvas', () {
      expect(() => repository().loadRefresh(-1), throwsRangeError);
      expect(() => repository().loadRefresh(sectionCount), throwsRangeError);
    });

    test('rejects accounts owned by another program', () async {
      await script.serveGame(sectionAccountOwner: systemProgramAddress.value);

      await expectLater(repository().loadRefresh(0), throwsStateError);
    });

    test('applies a matching colour canvas exactly once', () async {
      await script.serveGame(sectionPolicyMode: 1, sectionPolicyVersion: 3);
      var loads = 0;
      final colours = Uint8List(sectionPixelCount)..[0] = 4;

      final refresh = await repository(
        colourCanvasLoader: (gameIndex, sectionIndex, hint) async {
          loads++;

          return serverpod.ColourCanvasView(
            gameIndex: gameIndex,
            sectionIndex: sectionIndex,
            policyVersion: 3,
            highestRevision: 9,
            colours: ByteData.sublistView(colours),
          );
        },
      ).loadRefresh(0, colourLikely: true);

      expect(loads, 1, reason: 'the prefetched canvas must be reused');
      expect(
        refresh.snapshot?.section.colourMap?.codeAt(0),
        SectionColour.violet.code,
      );
      expect(refresh.snapshot?.section.colourRevision, 9);
    });

    test('ignores a colour canvas from another policy version', () async {
      await script.serveGame(sectionPolicyMode: 1, sectionPolicyVersion: 3);

      final refresh = await repository(
        colourCanvasLoader: (gameIndex, sectionIndex, hint) async {
          return serverpod.ColourCanvasView(
            gameIndex: gameIndex,
            sectionIndex: sectionIndex,
            policyVersion: 2,
            highestRevision: 8,
            colours: ByteData(sectionPixelCount),
          );
        },
      ).loadRefresh(0);

      expect(refresh.snapshot?.section.colourMap, isNull);
    });

    test('keeps chain state usable when the colour backend fails', () async {
      await script.serveGame(sectionPolicyMode: 1, sectionPolicyVersion: 3);

      final refresh = await repository(
        colourCanvasLoader: (gameIndex, sectionIndex, hint) async {
          throw StateError('rendering backend unavailable');
        },
      ).loadRefresh(0);

      expect(refresh.snapshot?.section.revision, BigInt.from(7));
      expect(refresh.snapshot?.section.colourMap, isNull);
    });

    test(
      'reuses the hinted canvas when the backend reports no change',
      () async {
        await script.serveGame(sectionPolicyMode: 1, sectionPolicyVersion: 3);
        final held = PixelColourMap.empty().painted(const [
          PixelCoordinate(2, 2),
        ], SectionColour.pink);
        ColourCanvasHint? receivedHint;

        final refresh =
            await repository(
              colourCanvasLoader: (gameIndex, sectionIndex, hint) async {
                receivedHint = hint;

                return serverpod.ColourCanvasView(
                  gameIndex: gameIndex,
                  sectionIndex: sectionIndex,
                  policyVersion: 3,
                  highestRevision: 11,
                  colours: null,
                );
              },
            ).loadRefresh(
              0,
              colourLikely: true,
              colourHint: ColourCanvasHint(
                policyVersion: 3,
                revision: 11,
                colourMap: held,
              ),
            );

        expect(receivedHint?.revision, 11);
        expect(receivedHint?.policyVersion, 3);
        expect(identical(refresh.snapshot?.section.colourMap, held), isTrue);
        expect(refresh.snapshot?.section.colourRevision, 11);
      },
    );

    test(
      'leaves the colour layer empty when nothing is held and unchanged',
      () async {
        await script.serveGame(sectionPolicyMode: 1, sectionPolicyVersion: 3);

        final refresh = await repository(
          colourCanvasLoader: (gameIndex, sectionIndex, hint) async {
            return serverpod.ColourCanvasView(
              gameIndex: gameIndex,
              sectionIndex: sectionIndex,
              policyVersion: 3,
              highestRevision: 0,
              colours: null,
            );
          },
        ).loadRefresh(0);

        expect(refresh.snapshot?.section.colourMap, isNull);
        expect(refresh.snapshot?.section.colourRevision, isNull);
      },
    );

    test('discards a wrong colour prefetch without loading twice', () async {
      await script.serveGame();
      var loads = 0;

      final refresh = await repository(
        colourCanvasLoader: (gameIndex, sectionIndex, hint) async {
          loads++;

          return serverpod.ColourCanvasView(
            gameIndex: gameIndex,
            sectionIndex: sectionIndex,
            policyVersion: 0,
            highestRevision: 0,
            colours: ByteData(sectionPixelCount),
          );
        },
      ).loadRefresh(0, colourLikely: true);

      expect(refresh.snapshot?.section.colourMap, isNull);
      expect(loads, 1, reason: 'the section is not in a colour round');
    });

    test(
      'falls back to the Serverpod client when no loader is injected',
      () async {
        await script.serveGame(sectionPolicyMode: 1, sectionPolicyVersion: 3);

        final repo = repository();
        final refresh = await repo.loadRefresh(0);
        final hinted = await repo.loadRefresh(
          0,
          colourHint: ColourCanvasHint(
            policyVersion: 3,
            revision: 1,
            colourMap: PixelColourMap.empty(),
          ),
        );

        expect(refresh.snapshot?.section.revision, BigInt.from(7));
        expect(
          refresh.snapshot?.section.colourMap,
          isNull,
          reason: 'no backend listens on the test URL, so the layer is skipped',
        );
        expect(hinted.snapshot?.section.colourMap, isNull);
      },
    );

    test('repeats the same batched fetch on every refresh', () async {
      await script.serveGame();
      final repo = repository();

      await repo.loadRefresh(0);
      await repo.loadRefresh(0);

      expect(script.methods, hasLength(2));
      expect(script.firstRequestedAddresses, script.requestedAddresses);
    });
  });

  group('SolanaBitflipRepository transactions', () {
    test(
      'lists a section through one signed and confirmed transaction',
      () async {
        final signature = await repository().listSection(
          _transactionSnapshot(),
          BigInt.from(5000),
        );

        expect(signature, _base58(64, 9));
        expect(wallet.signedTransactions, hasLength(1));
        expect(
          script.methods,
          containsAll(['getLatestBlockhash', 'getSignatureStatuses']),
        );
        expect(script.methods, isNot(contains('getMultipleAccounts')));
      },
    );

    test(
      'cancels a listing and seals through the memoized addresses',
      () async {
        final repo = repository();

        await repo.cancelSectionListing(_transactionSnapshot());
        await repo.sealSection(_transactionSnapshot());

        expect(wallet.signedTransactions, hasLength(2));
      },
    );

    test('claims a section and funds its vault in one transaction', () async {
      final signature = await repository().claimSection(
        _transactionSnapshot(sectionIndex: 1),
      );

      expect(signature, _base58(64, 9));
      expect(wallet.signedTransactions, hasLength(1));
    });

    test('flips pixels within the quoted reward window', () async {
      final signature = await repository().flipPixels(
        _transactionSnapshot(),
        const [PixelCoordinate(1, 2), PixelCoordinate(3, 4)],
        colour: null,
      );

      expect(signature, _base58(64, 9));
      expect(wallet.signedTransactions, hasLength(1));
    });
  });
}

const _config = BitflipConfig(
  environment: BitflipEnvironment.development,
  walletChain: 'solana:devnet',
  rpcUrl: 'http://localhost:8899/',
  serverpodUrl: 'http://localhost:8080/',
  gameIndex: 0,
);

/// A live, fully configured snapshot: every transaction builder finds the
/// treasury, BIT custody, and economy it requires.
GameSnapshot _transactionSnapshot({int sectionIndex = 0}) {
  final now = BigInt.from(DateTime.now().millisecondsSinceEpoch ~/ 1000);

  return GameSnapshot(
    gameIndex: 0,
    isDemo: false,
    nextSection: sectionIndex,
    totalFlips: BigInt.zero,
    mintedSections: 0,
    claimPriceLamports: BigInt.from(1_000_000),
    flipFeeLamports: BigInt.from(5000),
    startsAtUnixSeconds: BigInt.zero,
    unlockIntervalSeconds: 3600,
    earlyUnlockFlips: 1024,
    previousSectionFlipCount: BigInt.from(2048),
    treasury: _treasuryAddress,
    bitMint: _bitMintAddress,
    bitReserve: _bitReserveAddress,
    priceConfig: SectionPriceConfig(
      allocationTokens: BigInt.from(26_214_400),
      emissionDurationSeconds: BigInt.from(5_184_000),
      windowSeconds: BigInt.from(300),
      targetTokensPerWindow: BigInt.from(1024),
      startPriceLamports: BigInt.from(10_000),
      minimumPriceLamports: BigInt.from(5000),
      maximumPriceLamports: BigInt.from(1_000_000),
      startFloorPriceLamports: BigInt.from(5000),
      endFloorPriceLamports: BigInt.from(100_000),
      changeDenominator: BigInt.from(8),
      burstElasticity: BigInt.from(2),
    ),
    section: SectionSnapshot(
      index: sectionIndex,
      lifecycle: SectionLifecycle.active,
      bitmap: PixelBitmap.empty(),
      owner: _walletAddress,
      flipCount: BigInt.zero,
      revision: BigInt.zero,
      salePriceLamports: BigInt.from(5000),
      bitVault: _bitVaultAddress,
      economy: SectionEconomySnapshot(
        launchedAt: now,
        windowStartedAt: now,
        lastUpdatedAt: now,
        windowId: BigInt.zero,
        windowTargetTokens: BigInt.from(1024),
        windowRewardedTokens: BigInt.zero,
        emittedTokens: BigInt.zero,
        rewardPoolTokens: BigInt.zero,
        controllerPriceLamports: BigInt.from(10_000),
        postedPriceLamports: BigInt.from(10_000),
        protocolFeeLamports: BigInt.zero,
        ownerFeeLamports: BigInt.zero,
      ),
    ),
  );
}

/// Serves `getMultipleAccounts` from a fixed account map so the repository's
/// single-round-trip refresh can be asserted without a chain.
final class _ChainScript {
  final methods = <String>[];
  final accounts = <String, Map<String, Object?>>{};
  List<String> _firstRequested = const [];
  List<String> _lastRequested = const [];

  List<String> get requestedAddresses => _lastRequested;
  List<String> get firstRequestedAddresses => _firstRequested;

  Future<Object?> transport(RpcTransportConfig config) async {
    final payload = config.payload as Map<String, Object?>;
    final method = payload['method']! as String;
    final params = (payload['params'] ?? const <Object?>[]) as List<Object?>;
    methods.add(method);

    final result = switch (method) {
      'getMultipleAccounts' => _multipleAccounts(params),
      'getLatestBlockhash' => <String, Object?>{
        'context': _context,
        'value': <String, Object?>{
          'blockhash': _base58(32, 3),
          'lastValidBlockHeight': 1000,
        },
      },
      'getSignatureStatuses' => <String, Object?>{
        'context': _context,
        'value': [
          <String, Object?>{'confirmationStatus': 'confirmed', 'err': null},
        ],
      },
      'getEpochInfo' => <String, Object?>{
        'absoluteSlot': 1,
        'blockHeight': 10,
        'epoch': 0,
        'slotIndex': 1,
        'slotsInEpoch': 432000,
        'transactionCount': 1,
      },
      _ => throw StateError('Unexpected RPC method in test: $method'),
    };

    return <String, Object?>{
      'jsonrpc': '2.0',
      'id': payload['id'],
      'result': result,
    };
  }

  static const _context = <String, Object?>{'apiVersion': '2.0.0', 'slot': 1};

  Map<String, Object?> _multipleAccounts(List<Object?> params) {
    final requested = (params.first! as List<Object?>).cast<String>();
    _firstRequested = _firstRequested.isEmpty ? requested : _firstRequested;
    _lastRequested = requested;

    return <String, Object?>{
      'context': _context,
      'value': [for (final address in requested) accounts[address]],
    };
  }

  Future<void> serveGame({
    bool serveConfig = true,
    bool serveGameAccount = true,
    bool walletFunded = true,
    String? sectionAccountOwner,
    int sectionPolicyMode = 0,
    int sectionPolicyVersion = 0,
  }) async {
    final configAddress = (await findConfigPda(
      programAddress: bitflipProgramProgramAddress,
    )).$1;
    final gameAddress = (await findGamePda(
      programAddress: bitflipProgramProgramAddress,
      seeds: const GameSeeds(gameIndex: 0),
    )).$1;
    final sectionAddress = (await findSectionPda(
      programAddress: bitflipProgramProgramAddress,
      seeds: const SectionSeeds(gameIndex: 0, sectionIndex: 0),
    )).$1;

    if (serveConfig) {
      accounts[configAddress.value] = _encoded(
        getConfigStateEncoder().encode(_configState()),
      );
    }
    if (serveGameAccount) {
      accounts[gameAddress.value] = _encoded(
        getGameStateEncoder().encode(_gameState()),
      );
    }
    accounts[sectionAddress.value] = _encoded(
      getSectionStateEncoder().encode(
        _sectionState(
          owner: Address(_walletAddress),
          policyMode: sectionPolicyMode,
          policyVersion: BigInt.from(sectionPolicyVersion),
        ),
      ),
      owner: sectionAccountOwner,
    );
    accounts[_walletAddress] = _encoded(
      Uint8List(0),
      lamports: walletFunded ? 1234567890 : 0,
      owner: systemProgramAddress.value,
    );
  }

  Map<String, Object?> _encoded(
    Uint8List data, {
    int lamports = 0,
    String? owner,
  }) {
    return <String, Object?>{
      'data': [base64Encode(data), 'base64'],
      'executable': false,
      'lamports': lamports,
      'owner': owner ?? bitflipProgramProgramAddress.value,
      'rentEpoch': 0,
      'space': data.length,
    };
  }
}

ConfigState _configState() {
  return ConfigState(
    version: 1,
    authority: bitflipProgramProgramAddress,
    pendingAuthority: bitflipProgramProgramAddress,
    treasury: bitflipProgramProgramAddress,
    collectionAuthority: bitflipProgramProgramAddress,
    bitMint: systemProgramAddress,
    bitReserve: systemProgramAddress,
    claimPriceLamports: BigInt.from(1_000_000),
    flipFeeLamports: BigInt.from(5000),
    minimumFlipFeeLamports: BigInt.from(500),
    maximumFlipFeeLamports: BigInt.from(500_000),
    unlockIntervalSeconds: 3600,
    earlyUnlockFlips: 1024,
    gameCount: 1,
    bump: 254,
  );
}

GameState _gameState() {
  return GameState(
    gameIndex: 0,
    status: 1,
    bump: 254,
    economyVersion: 1,
    startsAt: BigInt.zero,
    nextSection: 1,
    mintedSections: 0,
    flipFeeLamports: BigInt.from(5000),
    totalFlips: BigInt.from(42),
    sectionAllocationTokens: BigInt.from(26_214_400),
    emissionDurationSeconds: BigInt.from(5_184_000),
    windowSeconds: BigInt.from(300),
    targetTokensPerWindow: BigInt.from(1024),
    startPriceLamports: BigInt.from(10_000),
    minimumPriceLamports: BigInt.from(5000),
    maximumPriceLamports: BigInt.from(1_000_000),
    startFloorPriceLamports: BigInt.from(5000),
    endFloorPriceLamports: BigInt.from(100_000),
    changeDenominator: BigInt.from(8),
    burstElasticity: BigInt.from(2),
    ownerShareBasisPoints: 2500,
  );
}

SectionState _sectionState({
  required Address owner,
  required int policyMode,
  required BigInt policyVersion,
}) {
  return SectionState(
    owner: owner,
    assetId: systemProgramAddress,
    merkleTree: systemProgramAddress,
    bitVault: systemProgramAddress,
    gameIndex: 0,
    sectionIndex: 0,
    status: 1,
    bump: 254,
    onPixels: 512,
    leafIndex: 0,
    flipCount: BigInt.from(12),
    revision: BigInt.from(7),
    lastFlipAt: BigInt.zero,
    salePriceLamports: BigInt.zero,
    economyLaunchedAt: BigInt.zero,
    economyWindowStartedAt: BigInt.zero,
    economyLastUpdatedAt: BigInt.zero,
    economyWindowId: BigInt.zero,
    economyWindowTargetTokens: BigInt.from(1024),
    economyWindowRewardedTokens: BigInt.zero,
    emittedTokens: BigInt.zero,
    rewardPoolTokens: BigInt.zero,
    controllerPriceLamports: BigInt.from(10_000),
    postedPriceLamports: BigInt.from(10_000),
    protocolFeeLamports: BigInt.zero,
    ownerFeeLamports: BigInt.zero,
    policyVersion: policyVersion,
    policyStartsAt: BigInt.zero,
    policyEndsAt: BigInt.zero,
    policyEntryPriceTokens: BigInt.zero,
    policyRewardPerActionTokens: BigInt.zero,
    policyRulesDigest: Uint8List(32),
    policyMode: policyMode,
    policyPaletteId: 0,
    policyRewardPolicy: 0,
    pixels: PixelBitmap.demo(2).bytes,
  );
}

/// A deterministic base58 string of [length] bytes all set to [fill].
String _base58(int length, int fill) {
  return getBase58Decoder().decode(
    Uint8List(length)..fillRange(0, length, fill),
  );
}

final class _FakeWallet implements BitflipWallet {
  _FakeWallet(this._address);

  final String? _address;
  final signedTransactions = <String>[];

  @override
  String? get address => _address;

  @override
  Future<void> initialize() async {}

  @override
  Future<String> signAndSend(String wireTransaction) async {
    signedTransactions.add(wireTransaction);

    return _base58(64, 9);
  }

  @override
  bool get isSupported => true;

  @override
  BitflipWalletKind get kind => BitflipWalletKind.embedded;

  @override
  bool get canFundWithMobileWallet => false;

  @override
  List<BitflipWalletOption>? get availableWallets => null;

  @override
  final String walletChain = 'solana:devnet';

  @override
  dynamic noSuchMethod(Invocation invocation) {
    throw StateError(
      'Unexpected wallet call in test: ${invocation.memberName}',
    );
  }
}
