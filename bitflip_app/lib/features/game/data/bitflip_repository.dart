import 'dart:typed_data';

import 'package:bitflip_app/core/bitflip_config.dart';
import 'package:bitflip_app/core/bitflip_wallet.dart';
import 'package:bitflip_app/features/game/domain/game_snapshot.dart';
import 'package:bitflip_app/features/game/domain/pixel_bitmap.dart';
import 'package:bitflip_program/bitflip_program.dart';
import 'package:bitflip_server_client/bitflip_server_client.dart' as serverpod;
import 'package:solana_kit/solana_kit.dart';
import 'package:solana_kit_rpc_spec/solana_kit_rpc_spec.dart';

abstract interface class BitflipRepository {
  bool get isDemoMode;
  bool get isWalletSupported;
  String get walletChain;
  List<BitflipWalletOption>? get availableWallets;
  String? get walletAddress;

  Future<GameSnapshot?> loadSection(int sectionIndex);
  Future<String> connectWallet([String? walletId]);
  Future<String> claimSection(GameSnapshot snapshot);
  Future<String> flipPixels(
    GameSnapshot snapshot,
    List<PixelCoordinate> coordinates,
  );
  Future<String> sealSection(GameSnapshot snapshot);
  Future<BitflipMintResult> mintSection(GameSnapshot snapshot);
}

final class BitflipMintResult {
  const BitflipMintResult({
    required this.assetId,
    required this.alreadyMinted,
    this.transactionSignature,
  });

  final String assetId;
  final String? transactionSignature;
  final bool alreadyMinted;
}

final class SolanaBitflipRepository implements BitflipRepository {
  SolanaBitflipRepository({
    required BitflipConfig config,
    BitflipWallet? wallet,
  }) : _config = config,
       _gameIndex = config.gameIndex,
       _rpc = createSolanaRpc(
         url: config.rpcUrl,
         allowInsecureHttp: _isLoopback(config.rpcUrl),
       ),
       _serverpod = serverpod.Client(config.serverpodUrl),
       _wallet = wallet ?? BitflipWallet(walletChain: config.walletChain);

  final BitflipConfig _config;
  final int _gameIndex;
  final Rpc _rpc;
  final serverpod.Client _serverpod;
  final BitflipWallet _wallet;

  @override
  bool get isDemoMode => false;

  @override
  bool get isWalletSupported => _wallet.isSupported;

  @override
  String get walletChain => _config.walletChain;

  @override
  List<BitflipWalletOption>? get availableWallets => _wallet.availableWallets;

  @override
  String? get walletAddress => _wallet.address;

  @override
  Future<GameSnapshot?> loadSection(int sectionIndex) async {
    if (sectionIndex < 0 || sectionIndex >= sectionCount) {
      throw RangeError.range(sectionIndex, 0, sectionCount - 1);
    }
    final (configAddress, _) = await findConfigPda(
      programAddress: bitflipProgramProgramAddress,
    );
    final (gameAddress, _) = await findGamePda(
      programAddress: bitflipProgramProgramAddress,
      seeds: GameSeeds(gameIndex: _gameIndex),
    );
    final (sectionAddress, _) = await findSectionPda(
      programAddress: bitflipProgramProgramAddress,
      seeds: SectionSeeds(gameIndex: _gameIndex, sectionIndex: sectionIndex),
    );
    final accounts = await fetchEncodedAccounts(_rpc, [
      configAddress,
      gameAddress,
      sectionAddress,
    ]);
    final configAccount = _existingAccount(accounts[0]);
    final gameAccount = _existingAccount(accounts[1]);
    if (configAccount == null || gameAccount == null) {
      return null;
    }
    _assertProgramOwner(configAccount);
    _assertProgramOwner(gameAccount);
    final config = decodeConfigState(configAccount.account).data;
    final game = decodeGameState(gameAccount.account).data;
    final encodedSection = _existingAccount(accounts[2]);
    final section = encodedSection == null
        ? SectionSnapshot(
            index: sectionIndex,
            lifecycle: SectionLifecycle.unclaimed,
            bitmap: PixelBitmap.empty(),
            owner: null,
            flipCount: BigInt.zero,
            revision: BigInt.zero,
          )
        : _decodeSection(encodedSection);

    return GameSnapshot(
      gameIndex: game.gameIndex,
      isDemo: false,
      nextSection: game.nextSection,
      totalFlips: game.totalFlips,
      mintedSections: game.mintedSections,
      claimPriceLamports: config.claimPriceLamports,
      flipFeeLamports: game.flipFeeLamports,
      treasury: config.treasury.value,
      section: section,
    );
  }

  @override
  Future<String> connectWallet([String? walletId]) => _wallet.connect(walletId);

  @override
  Future<String> claimSection(GameSnapshot snapshot) async {
    final owner = _requireWalletAddress();
    final treasury = _requireTreasury(snapshot);
    final sectionIndex = snapshot.section.index;
    final (config, _) = await findConfigPda(
      programAddress: bitflipProgramProgramAddress,
    );
    final (game, _) = await findGamePda(
      programAddress: bitflipProgramProgramAddress,
      seeds: GameSeeds(gameIndex: snapshot.gameIndex),
    );
    final (section, bump) = await findSectionPda(
      programAddress: bitflipProgramProgramAddress,
      seeds: SectionSeeds(
        gameIndex: snapshot.gameIndex,
        sectionIndex: sectionIndex,
      ),
    );
    final previousSection = sectionIndex == 0
        ? systemProgramAddress
        : (await findSectionPda(
            programAddress: bitflipProgramProgramAddress,
            seeds: SectionSeeds(
              gameIndex: snapshot.gameIndex,
              sectionIndex: sectionIndex - 1,
            ),
          )).$1;
    final instruction = getClaimSectionInstruction(
      programAddress: bitflipProgramProgramAddress,
      owner: owner,
      config: config,
      game: game,
      previousSection: previousSection,
      section: section,
      treasury: treasury,
      systemProgram: systemProgramAddress,
      gameIndex: snapshot.gameIndex,
      sectionIndex: sectionIndex,
      bump: bump,
      maximumPriceLamports: snapshot.claimPriceLamports,
    );
    return _send(instruction, owner);
  }

  @override
  Future<String> flipPixels(
    GameSnapshot snapshot,
    List<PixelCoordinate> coordinates,
  ) async {
    if (coordinates.isEmpty || coordinates.length > maxFlipBatch) {
      throw ArgumentError.value(coordinates.length, 'coordinates.length');
    }
    if (coordinates.toSet().length != coordinates.length) {
      throw ArgumentError('Pixel coordinates must be unique.');
    }
    final player = _requireWalletAddress();
    final treasury = _requireTreasury(snapshot);
    final (config, _) = await findConfigPda(
      programAddress: bitflipProgramProgramAddress,
    );
    final (game, _) = await findGamePda(
      programAddress: bitflipProgramProgramAddress,
      seeds: GameSeeds(gameIndex: snapshot.gameIndex),
    );
    final (section, _) = await findSectionPda(
      programAddress: bitflipProgramProgramAddress,
      seeds: SectionSeeds(
        gameIndex: snapshot.gameIndex,
        sectionIndex: snapshot.section.index,
      ),
    );
    final packedCoordinates = Uint8List(32);
    for (var index = 0; index < coordinates.length; index++) {
      packedCoordinates[index * 2] = coordinates[index].x;
      packedCoordinates[index * 2 + 1] = coordinates[index].y;
    }
    final maximumTotalFee =
        snapshot.flipFeeLamports * BigInt.from(coordinates.length);
    final instruction = getFlipPixelsInstruction(
      programAddress: bitflipProgramProgramAddress,
      player: player,
      config: config,
      game: game,
      section: section,
      treasury: treasury,
      systemProgram: systemProgramAddress,
      gameIndex: snapshot.gameIndex,
      sectionIndex: snapshot.section.index,
      count: coordinates.length,
      coordinates: packedCoordinates,
      maximumTotalFeeLamports: maximumTotalFee,
    );
    return _send(instruction, player);
  }

  @override
  Future<String> sealSection(GameSnapshot snapshot) async {
    final owner = _requireWalletAddress();
    final (game, _) = await findGamePda(
      programAddress: bitflipProgramProgramAddress,
      seeds: GameSeeds(gameIndex: snapshot.gameIndex),
    );
    final (section, _) = await findSectionPda(
      programAddress: bitflipProgramProgramAddress,
      seeds: SectionSeeds(
        gameIndex: snapshot.gameIndex,
        sectionIndex: snapshot.section.index,
      ),
    );
    final instruction = getSealSectionInstruction(
      programAddress: bitflipProgramProgramAddress,
      owner: owner,
      game: game,
      section: section,
      gameIndex: snapshot.gameIndex,
      sectionIndex: snapshot.section.index,
    );
    return _send(instruction, owner);
  }

  @override
  Future<BitflipMintResult> mintSection(GameSnapshot snapshot) async {
    final walletAddress = _requireWalletAddress().value;
    final challenge = await _serverpod.mint.createChallenge(
      walletAddress: walletAddress,
      gameIndex: snapshot.gameIndex,
      sectionIndex: snapshot.section.index,
    );
    final signature = await _wallet.signMessage(challenge.message);
    final result = await _serverpod.mint.mintSection(
      walletAddress: walletAddress,
      gameIndex: snapshot.gameIndex,
      sectionIndex: snapshot.section.index,
      nonce: challenge.nonce,
      signatureBase64: signature,
    );
    return BitflipMintResult(
      assetId: result.assetId,
      transactionSignature: result.transactionSignature,
      alreadyMinted: result.alreadyMinted,
    );
  }

  Future<String> _send(Instruction instruction, Address feePayer) async {
    final latest = await _rpc.getLatestBlockhashValue().send();
    final transaction = compileTransaction(
      createTransactionMessage(version: TransactionVersion.v0)
          .withFeePayer(feePayer)
          .withBlockhashLifetime(
            BlockhashLifetimeConstraint(
              blockhash: latest.value.blockhash.value,
              lastValidBlockHeight: latest.value.lastValidBlockHeight,
            ),
          )
          .appendInstructions([instruction]),
    );
    final transactionSignature = signature(
      await _wallet.signAndSend(getBase64EncodedWireTransaction(transaction)),
    );
    await waitForTransactionConfirmation(
      rpc: _rpc,
      signature: transactionSignature,
      transaction: transaction,
      config: const RpcTransactionConfirmationConfig(
        commitment: Commitment.confirmed,
        pollInterval: Duration(milliseconds: 400),
      ),
    );
    return transactionSignature.value;
  }

  Address _requireWalletAddress() {
    final address = walletAddress;
    if (address == null) {
      throw StateError('Connect a wallet before signing.');
    }
    return Address(address);
  }

  static Address _requireTreasury(GameSnapshot snapshot) {
    final treasury = snapshot.treasury;
    if (treasury == null) {
      throw StateError('The current game has no treasury configuration.');
    }
    return Address(treasury);
  }

  static ExistingAccount<Uint8List>? _existingAccount(
    MaybeEncodedAccount account,
  ) {
    return switch (account) {
      ExistingAccount<Uint8List>() => account,
      NonExistingAccount<Uint8List>() => null,
    };
  }

  static void _assertProgramOwner(ExistingAccount<Uint8List> account) {
    if (account.programAddress != bitflipProgramProgramAddress) {
      throw StateError('A Bitflip PDA is owned by an unexpected program.');
    }
  }

  static SectionSnapshot _decodeSection(ExistingAccount<Uint8List> account) {
    _assertProgramOwner(account);
    final data = decodeSectionState(account.account).data;
    final lifecycle = switch (data.status) {
      1 => SectionLifecycle.active,
      2 => SectionLifecycle.sealed,
      3 => SectionLifecycle.minted,
      _ => throw StateError('The section has an invalid lifecycle.'),
    };
    return SectionSnapshot(
      index: data.sectionIndex,
      lifecycle: lifecycle,
      bitmap: PixelBitmap.fromBytes(data.pixels),
      owner: data.owner.value,
      flipCount: data.flipCount,
      revision: data.revision,
      assetId: lifecycle == SectionLifecycle.minted ? data.assetId.value : null,
    );
  }
}

bool _isLoopback(String value) {
  final host = Uri.tryParse(value)?.host.toLowerCase();
  return host == '127.0.0.1' || host == 'localhost' || host == '::1';
}
