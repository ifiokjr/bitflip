@TestOn('vm')
@Tags(['surfpool'])
library;

import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:bitflip_program/bitflip_program.dart';
import 'package:bitflip_server_server/src/minting/bitflip_mint_service.dart';
import 'package:solana_kit/solana_kit.dart';
import 'package:solana_kit_mpl_bubblegum/solana_kit_mpl_bubblegum.dart'
    as bubblegum;
import 'package:solana_kit_rpc_api/solana_kit_rpc_api.dart' as rpc_api;
import 'package:solana_kit_rpc_types/solana_kit_rpc_types.dart' as rpc_types;
import 'package:solana_kit_surfpool/solana_kit_surfpool.dart';
import 'package:solana_kit_system/solana_kit_system.dart' as system;
import 'package:test/test.dart';

const _bubblegumProgram = Address(
  'BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY',
);
const _compressionProgram = Address(
  'cmtDvXumGCrqC1Age74AVPhSRVXJMd8PJS91L8KbNCK',
);
const _noopProgram = Address('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV');
const _disableSbpfV0V1V2DeploymentFeature = Address(
  'B8JJXCy5amZyWG9r7EnUYLwzXSXTxG7GZ1qZ1qggo83g',
);

void main() {
  late SurfpoolClient surfpool;
  late Uint8List operatorSecretKey;
  late KeyPairSigner owner;
  late KeyPairSigner merkleTree;
  late Address configAddress;
  late Address gameAddress;
  late Address sectionAddress;

  setUpAll(() async {
    surfpool = await createSurfpoolClient(
      config: SurfnetConfig(
        offline: true,
        blockProductionMode: BlockProductionMode.clock,
        slotTimeMs: 10,
        disableFeatures: const [_disableSbpfV0V1V2DeploymentFeature],
      ),
    );
    operatorSecretKey = surfpool.surfnet.payerSecretKey;
    owner = generateKeyPairSigner();
    merkleTree = generateKeyPairSigner();

    await Future.wait([
      _deployProgram(
        surfpool,
        bitflipProgramProgramAddress,
        _rootPath('target/deploy/bitflip_program.so'),
      ),
      _deployProgram(
        surfpool,
        _bubblegumProgram,
        _artifactPath('mpl_bubblegum-v0.12.0.so'),
      ),
      _deployProgram(
        surfpool,
        _compressionProgram,
        _artifactPath('spl_account_compression-v0.3.3.so'),
      ),
      _deployProgram(surfpool, _noopProgram, _artifactPath('noop-v0.2.0.so')),
    ]);
    await surfpool.airdrop(owner.address, BigInt.from(2_000_000_000));

    final (derivedConfigAddress, configBump) = await findConfigPda(
      programAddress: bitflipProgramProgramAddress,
    );
    final (derivedGameAddress, gameBump) = await findGamePda(
      programAddress: bitflipProgramProgramAddress,
      seeds: const GameSeeds(gameIndex: 0),
    );
    final (initialSectionAddress, initialSectionBump) = await findSectionPda(
      programAddress: bitflipProgramProgramAddress,
      seeds: const SectionSeeds(gameIndex: 0, sectionIndex: 0),
    );
    final (derivedSectionAddress, sectionBump) = await findSectionPda(
      programAddress: bitflipProgramProgramAddress,
      seeds: const SectionSeeds(gameIndex: 0, sectionIndex: 1),
    );
    configAddress = derivedConfigAddress;
    gameAddress = derivedGameAddress;
    sectionAddress = derivedSectionAddress;

    await _sendInstructions(surfpool, [
      getInitializeConfigInstruction(
        programAddress: bitflipProgramProgramAddress,
        payer: surfpool.payer.address,
        config: configAddress,
        systemProgram: system.systemProgramAddress,
        bump: configBump,
      ),
    ]);
    await _setTestConfigAuthorities(surfpool, configAddress);
    await _sendInstructions(surfpool, [
      getInitializeGameInstruction(
        programAddress: bitflipProgramProgramAddress,
        payer: surfpool.payer.address,
        config: configAddress,
        game: gameAddress,
        section: initialSectionAddress,
        systemProgram: system.systemProgramAddress,
        gameIndex: 0,
        sectionIndex: 0,
        gameBump: gameBump,
        sectionBump: initialSectionBump,
      ),
    ]);
    final coordinates = Uint8List(32)
      ..[0] = 7
      ..[1] = 9;
    await _sendInstructions(
      surfpool,
      [
        getFlipPixelsInstruction(
          programAddress: bitflipProgramProgramAddress,
          player: owner.address,
          config: configAddress,
          game: gameAddress,
          section: initialSectionAddress,
          treasury: surfpool.payer.address,
          systemProgram: system.systemProgramAddress,
          gameIndex: 0,
          sectionIndex: 0,
          count: 1,
          coordinates: coordinates,
          maximumTotalFeeLamports: BigInt.from(10_000),
        ),
        getClaimSectionInstruction(
          programAddress: bitflipProgramProgramAddress,
          owner: owner.address,
          config: configAddress,
          game: gameAddress,
          previousSection: initialSectionAddress,
          section: sectionAddress,
          treasury: surfpool.payer.address,
          systemProgram: system.systemProgramAddress,
          gameIndex: 0,
          sectionIndex: 1,
          bump: sectionBump,
          maximumPriceLamports: BigInt.from(10_000_000),
        ),
        getFlipPixelsInstruction(
          programAddress: bitflipProgramProgramAddress,
          player: owner.address,
          config: configAddress,
          game: gameAddress,
          section: sectionAddress,
          treasury: surfpool.payer.address,
          systemProgram: system.systemProgramAddress,
          gameIndex: 0,
          sectionIndex: 1,
          count: 1,
          coordinates: coordinates,
          maximumTotalFeeLamports: BigInt.from(10_000),
        ),
        getSealSectionInstruction(
          programAddress: bitflipProgramProgramAddress,
          owner: owner.address,
          game: gameAddress,
          section: sectionAddress,
          gameIndex: 0,
          sectionIndex: 1,
        ),
      ],
      extraSigners: [owner],
    );
    await _createPrivateTree(surfpool, merkleTree);
  });

  tearDownAll(() async {
    owner.keyPair.dispose();
    merkleTree.keyPair.dispose();
    await surfpool.stop();
  });

  test('deploys the Bitflip and Bubblegum program stack', () async {
    for (final program in const [
      bitflipProgramProgramAddress,
      _bubblegumProgram,
      _compressionProgram,
      _noopProgram,
    ]) {
      final response = await surfpool.rpc.getAccountInfoValue(program).send();
      expect(response.value, isNotNull, reason: '$program should exist');
      expect(response.value!['executable'], isTrue);
    }
  });

  test(
    'atomically mints a sealed section and records its Pina receipt',
    () async {
      final service = SolanaBitflipMintService(
        rpc: surfpool.rpc,
        merkleTreeAddress: merkleTree.address.value,
        operatorPrivateKey: jsonEncode(operatorSecretKey),
        metadataBaseUrl: 'https://bitflip.invalid',
      );
      final before = await service.loadSection(0, 1);
      expect(before.isSealed, isTrue);
      expect(before.owner, owner.address);
      expect(before.bitmap[72], 128, reason: 'pixel (7, 9) should be on');

      final result = await service.mint(before);
      expect(result.alreadyMinted, isFalse);
      expect(result.transactionSignature, isNotEmpty);
      expect(result.merkleTree, merkleTree.address);
      expect(result.leafIndex, 0);

      final after = await service.loadSection(0, 1);
      expect(after.isMinted, isTrue);
      expect(after.assetId, result.assetId);
      expect(after.merkleTree, merkleTree.address);
      expect(after.leafIndex, 0);

      final (treeAuthority, _) = await bubblegum.findTreeAuthorityPda(
        merkleTree: merkleTree.address,
      );
      final treeConfig = await fetchEncodedAccount(surfpool.rpc, treeAuthority);
      final data = switch (treeConfig) {
        ExistingAccount<Uint8List>(:final data) => data,
        NonExistingAccount<Uint8List>() => fail('Tree config should exist.'),
      };
      expect(data.length, 96);
      expect(data[88], 0, reason: 'Bitflip trees must be private');

      final transaction = await surfpool.rpc
          .getTransaction(
            Signature(result.transactionSignature!),
            const rpc_api.GetTransactionConfig(
              commitment: rpc_types.Commitment.confirmed,
              maxSupportedTransactionVersion: 0,
            ),
          )
          .send();
      expect(transaction, isNotNull);
      final meta = transaction!['meta']! as Map<String, Object?>;
      expect(meta['err'], isNull);
      final logs = (meta['logMessages']! as List).cast<String>();
      expect(logs, contains('Program ${_bubblegumProgram.value} success'));
      expect(
        logs,
        contains('Program ${bitflipProgramProgramAddress.value} success'),
      );
    },
  );

  test('returns the recorded receipt without minting twice', () async {
    final service = SolanaBitflipMintService(
      rpc: surfpool.rpc,
      merkleTreeAddress: merkleTree.address.value,
      operatorPrivateKey: jsonEncode(operatorSecretKey),
      metadataBaseUrl: 'https://bitflip.invalid',
    );
    final minted = await service.loadSection(0, 1);
    final result = await service.mint(minted);

    expect(result.alreadyMinted, isTrue);
    expect(result.transactionSignature, isNull);
    expect(result.assetId, minted.assetId);
    expect(result.leafIndex, 0);
  });
}

Future<void> _createPrivateTree(
  SurfpoolClient client,
  KeyPairSigner merkleTree,
) async {
  const maxDepth = 3;
  const maxBufferSize = 8;
  final space = _concurrentMerkleTreeAccountSize(
    maxDepth: maxDepth,
    maxBufferSize: maxBufferSize,
  );
  final rent = await client.getMinimumBalance(BigInt.from(space));
  final (treeAuthority, _) = await bubblegum.findTreeAuthorityPda(
    merkleTree: merkleTree.address,
  );

  await _sendInstructions(
    client,
    [
      system.getCreateAccountInstruction(
        instructionProgramAddress: system.systemProgramAddress,
        payer: client.payer.address,
        newAccount: merkleTree.address,
        lamports: rent,
        space: BigInt.from(space),
        programAddress: _compressionProgram,
      ),
      bubblegum.getCreateTreeInstruction(
        programAddress: _bubblegumProgram,
        treeAuthority: treeAuthority,
        merkleTree: merkleTree.address,
        payer: client.payer.address,
        treeCreator: client.payer.address,
        logWrapper: _noopProgram,
        compressionProgram: _compressionProgram,
        systemProgram: system.systemProgramAddress,
        maxDepth: maxDepth,
        maxBufferSize: maxBufferSize,
        public: false,
      ),
    ],
    extraSigners: [merkleTree],
  );
}

Future<void> _setTestConfigAuthorities(
  SurfpoolClient client,
  Address configAddress,
) async {
  final maybeConfig = await fetchEncodedAccount(client.rpc, configAddress);
  final config = switch (maybeConfig) {
    ExistingAccount<Uint8List>(:final account) => account,
    NonExistingAccount<Uint8List>() => throw StateError(
      'Bitflip config should exist before test setup.',
    ),
  };
  final data = Uint8List.fromList(config.data);
  final authority = getAddressEncoder().encode(client.payer.address);
  data
    ..setRange(2, 34, authority)
    ..setRange(66, 98, authority)
    ..setRange(98, 130, authority)
    ..setRange(230, 234, const [1, 0, 0, 0]);
  await client.cheatcodes.setAccount(configAddress, data: data);
}

Future<String> _sendInstructions(
  SurfpoolClient client,
  List<Instruction> instructions, {
  List<KeyPairSigner> extraSigners = const [],
}) async {
  final latest = await client.rpc.getLatestBlockhashValue().send();
  final transaction = compileTransaction(
    createTransactionMessage(version: TransactionVersion.v0)
        .withFeePayer(client.payer.address)
        .withBlockhashLifetime(
          BlockhashLifetimeConstraint(
            blockhash: latest.value.blockhash.value,
            lastValidBlockHeight: latest.value.lastValidBlockHeight,
          ),
        )
        .appendInstructions(instructions),
  );
  final signed = await signTransactionWithSigners([
    client.payer,
    ...extraSigners,
  ], transaction);
  late final Signature transactionSignature;
  try {
    transactionSignature = signature(
      await client.rpc
          .sendTransaction(
            getBase64EncodedWireTransaction(signed),
            const rpc_api.SendTransactionConfig(
              encoding: rpc_types.WireTransactionEncoding.base64,
              preflightCommitment: rpc_types.Commitment.confirmed,
            ),
          )
          .send(),
    );
  } on SolanaError catch (error) {
    throw StateError('Surfpool transaction failed: ${error.context}');
  }
  await waitForTransactionConfirmation(
    rpc: client.rpc,
    signature: transactionSignature,
    transaction: signed,
  );
  return transactionSignature.value;
}

Future<void> _deployProgram(
  SurfpoolClient client,
  Address program,
  String path,
) {
  if (!File(path).existsSync()) {
    throw StateError('Missing Surfpool program artifact: $path');
  }
  return client.surfnet.deploy(DeployOptions(programId: program, soPath: path));
}

String _artifactPath(String name) =>
    _rootPath('.tools/surfpool-programs/$name');

String _rootPath(String path) => File('../../$path').absolute.path;

int _concurrentMerkleTreeAccountSize({
  required int maxDepth,
  required int maxBufferSize,
}) {
  const headerSize = 56;
  const treePrefixSize = 24;
  final changeLogSize = 40 + (32 * maxDepth);
  final rightMostPathSize = 40 + (32 * maxDepth);
  return headerSize +
      treePrefixSize +
      (maxBufferSize * changeLogSize) +
      rightMostPathSize;
}
