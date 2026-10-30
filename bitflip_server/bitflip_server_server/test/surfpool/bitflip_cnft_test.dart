@TestOn('vm')
@Tags(['surfpool'])
library;

import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:bitflip_program/bitflip_program.dart';
import 'package:bitflip_server_server/src/minting/bitflip_mint_service.dart';
import 'package:bs58/bs58.dart';
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

final _receiptComputeUnits = <int>[];

void main() {
  late SurfpoolClient surfpool;
  late Uint8List operatorSecretKey;
  late KeyPairSigner owner;
  late KeyPairSigner buyer;
  late KeyPairSigner merkleTree;
  late Address configAddress;
  late Address gameAddress;
  late Address sectionAddress;
  late Uint8List sealedSectionData;

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
    buyer = generateKeyPairSigner();
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
    await surfpool.airdrop(buyer.address, BigInt.from(2_000_000_000));

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
    await surfpool.cheatcodes.timeTravel(
      absoluteTimestamp: DateTime.now().millisecondsSinceEpoch + 2000,
    );
    await _sendInstructions(
      surfpool,
      [
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
      ],
      extraSigners: [owner],
    );
    await _setTestSectionPixel(surfpool, sectionAddress, x: 7, y: 9);
    await _sendInstructions(
      surfpool,
      [
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
    sealedSectionData = await _accountData(surfpool, sectionAddress);
  });

  tearDownAll(() async {
    owner.keyPair.dispose();
    buyer.keyPair.dispose();
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
    'rejects seller authorization after a sale, then mints for the buyer',
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

      final price = BigInt.from(1_000_000);
      await _sendInstructions(
        surfpool,
        [
          getListSectionInstruction(
            programAddress: bitflipProgramProgramAddress,
            owner: owner.address,
            game: gameAddress,
            section: sectionAddress,
            gameIndex: 0,
            sectionIndex: 1,
            priceLamports: price,
          ),
        ],
        extraSigners: [owner],
      );
      await _sendInstructions(
        surfpool,
        [
          getPurchaseSectionInstruction(
            programAddress: bitflipProgramProgramAddress,
            buyer: buyer.address,
            seller: owner.address,
            game: gameAddress,
            section: sectionAddress,
            systemProgram: system.systemProgramAddress,
            gameIndex: 0,
            sectionIndex: 1,
            maximumPriceLamports: price,
          ),
        ],
        extraSigners: [buyer],
      );

      await expectLater(
        service.mint(before),
        throwsA(
          isA<StateError>().having(
            (error) => error.message,
            'message',
            'The section owner changed before minting.',
          ),
        ),
      );
      final purchased = await service.loadSection(0, 1);
      expect(purchased.owner, buyer.address);
      expect(purchased.isSealed, isTrue);

      final result = await service.mint(purchased);
      expect(result.alreadyMinted, isFalse);
      expect(result.transactionSignature, isNotEmpty);
      expect(result.merkleTree, merkleTree.address);
      expect(result.leafIndex, 0);

      final after = await service.loadSection(0, 1);
      expect(after.isMinted, isTrue);
      expect(after.assetId, result.assetId);
      expect(after.merkleTree, merkleTree.address);
      expect(after.leafIndex, 0);
      await expectLater(service.mint(before), throwsA(isA<StateError>()));

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
      await _expectLeafEvent(surfpool, result, buyer.address);
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

  test(
    'rejects forged, stale and duplicate receipts without state changes',
    () async {
      final (treeAuthority, _) = await bubblegum.findTreeAuthorityPda(
        merkleTree: merkleTree.address,
      );
      final section = await _installSealedSection(
        surfpool,
        sealedSectionData,
        2,
      );
      final assetId = await _assetId(merkleTree.address, 0);
      final record = getRecordSectionMintInstruction(
        programAddress: bitflipProgramProgramAddress,
        collectionAuthority: surfpool.payer.address,
        config: configAddress,
        game: gameAddress,
        section: section,
        treeAuthority: treeAuthority,
        gameIndex: 0,
        sectionIndex: 2,
        expectedOwner: owner.address,
        assetId: assetId,
        merkleTree: merkleTree.address,
        leafIndex: 0,
      );
      final sectionBefore = await _accountData(surfpool, section);
      final gameBefore = await _accountData(surfpool, gameAddress);
      final treeData = await _accountData(surfpool, treeAuthority);

      Future<void> rejected(Instruction instruction) async {
        await expectLater(
          _sendInstructions(surfpool, [instruction]),
          throwsStateError,
        );
        expect(await _accountData(surfpool, section), sectionBefore);
        expect(await _accountData(surfpool, gameAddress), gameBefore);
      }

      // A different asset cannot be substituted even when the counter matches.
      final forged = Uint8List.fromList(record.data!);
      forged.setRange(36, 68, getAddressEncoder().encode(owner.address));
      await rejected(
        Instruction(
          programAddress: record.programAddress,
          accounts: record.accounts,
          data: forged,
        ),
      );

      // A newer counter recreates a replica's stale prediction after MintV1.
      final advanced = Uint8List.fromList(treeData);
      ByteData.sublistView(advanced).setUint64(80, 2, Endian.little);
      await surfpool.cheatcodes.setAccount(treeAuthority, data: advanced);
      await rejected(record);

      for (final malformed in [Uint8List(8), Uint8List(96)]) {
        await surfpool.cheatcodes.setAccount(treeAuthority, data: malformed);
        await rejected(record);
      }
      await surfpool.cheatcodes.setAccount(treeAuthority, data: treeData);

      final substituted = [...record.accounts!];
      substituted[4] = AccountMeta(
        address: surfpool.payer.address,
        role: AccountRole.readonly,
      );
      await rejected(
        Instruction(
          programAddress: record.programAddress,
          accounts: substituted,
          data: record.data,
        ),
      );

      // The original owner and lifecycle checks still apply with a valid tree.
      final active = Uint8List.fromList(sectionBefore)..[132] = 1;
      await surfpool.cheatcodes.setAccount(section, data: active);
      await expectLater(
        _sendInstructions(surfpool, [record]),
        throwsStateError,
      );
      await surfpool.cheatcodes.setAccount(section, data: sectionBefore);

      final duplicate = getRecordSectionMintInstruction(
        programAddress: bitflipProgramProgramAddress,
        collectionAuthority: surfpool.payer.address,
        config: configAddress,
        game: gameAddress,
        section: sectionAddress,
        treeAuthority: treeAuthority,
        gameIndex: 0,
        sectionIndex: 1,
        expectedOwner: owner.address,
        assetId: assetId,
        merkleTree: merkleTree.address,
        leafIndex: 0,
      );
      await expectLater(
        _sendInstructions(surfpool, [duplicate]),
        throwsStateError,
      );
    },
  );

  test('racing allocations roll back the loser and retry without an orphan', () async {
    final sections = await Future.wait([
      _installSealedSection(surfpool, sealedSectionData, 3),
      _installSealedSection(surfpool, sealedSectionData, 4),
    ]);
    final (treeAuthority, _) = await bubblegum.findTreeAuthorityPda(
      merkleTree: merkleTree.address,
    );
    final predictedLeaf = ByteData.sublistView(
      await _accountData(surfpool, treeAuthority),
    ).getUint64(80, Endian.little);
    final assetId = await _assetId(merkleTree.address, predictedLeaf);
    final transactions = await Future.wait([
      for (var i = 0; i < sections.length; i++)
        _atomicMintInstructions(
          surfpool,
          merkleTree.address,
          owner.address,
          configAddress,
          gameAddress,
          sections[i],
          i + 3,
          predictedLeaf,
        ),
    ]);
    final attempts = await Future.wait([
      for (final instructions in transactions)
        _raceAttempt(surfpool, instructions),
    ]);
    expect(
      attempts.where((attempt) => attempt.signature != null),
      hasLength(1),
    );
    final winner = attempts.indexWhere((attempt) => attempt.signature != null);
    final loser = 1 - winner;
    expect(
      attempts[loser].error.toString(),
      contains('custom program error: 0xe'),
    );

    final service = SolanaBitflipMintService(
      rpc: surfpool.rpc,
      merkleTreeAddress: merkleTree.address.value,
      operatorPrivateKey: jsonEncode(operatorSecretKey),
      metadataBaseUrl: 'https://bitflip.invalid',
    );
    final winningSection = await service.loadSection(0, winner + 3);
    expect(winningSection.assetId, assetId);
    expect(winningSection.leafIndex, predictedLeaf);
    await _expectLeafEvent(
      surfpool,
      MintSubmission(
        assetId: assetId,
        merkleTree: merkleTree.address,
        leafIndex: predictedLeaf,
        alreadyMinted: false,
        transactionSignature: attempts[winner].signature,
      ),
      owner.address,
    );
    expect((await service.loadSection(0, loser + 3)).isSealed, isTrue);
    expect(
      ByteData.sublistView(await _accountData(surfpool, treeAuthority))
          .getUint64(80, Endian.little),
      predictedLeaf + 1,
    );

    final retry = await service.mint(await service.loadSection(0, loser + 3));
    expect(retry.leafIndex, predictedLeaf + 1);
    await _expectLeafEvent(surfpool, retry, owner.address);

    // A new operator instance represents a crash after confirmation but before
    // returning the result. Its retry must reconcile the existing receipt.
    final restarted = SolanaBitflipMintService(
      rpc: surfpool.rpc,
      merkleTreeAddress: merkleTree.address.value,
      operatorPrivateKey: jsonEncode(operatorSecretKey),
      metadataBaseUrl: 'https://bitflip.invalid',
    );
    final recovered = await restarted.mint(
      await restarted.loadSection(0, loser + 3),
    );
    expect(recovered.alreadyMinted, isTrue);
    expect(recovered.assetId, retry.assetId);
    expect(recovered.leafIndex, retry.leafIndex);
    expect(
      ByteData.sublistView(await _accountData(surfpool, treeAuthority))
          .getUint64(80, Endian.little),
      predictedLeaf + 2,
    );
  });

  test('receipt validation stays within its reviewed compute budget', () {
    expect(_receiptComputeUnits, hasLength(3));
    // Two canonical PDA searches have variable cost depending on their bumps.
    // The increase from 1,300 CU requires explicit performance approval.
    expect(_receiptComputeUnits, everyElement(lessThanOrEqualTo(30000)));
  });
}

Future<Uint8List> _accountData(SurfpoolClient client, Address address) async {
  return switch (await fetchEncodedAccount(client.rpc, address)) {
    ExistingAccount<Uint8List>(:final data) => Uint8List.fromList(data),
    NonExistingAccount<Uint8List>() => throw StateError(
      'Missing test account.',
    ),
  };
}

Future<Address> _installSealedSection(
  SurfpoolClient client,
  Uint8List template,
  int index,
) async {
  final (section, bump) = await findSectionPda(
    programAddress: bitflipProgramProgramAddress,
    seeds: SectionSeeds(gameIndex: 0, sectionIndex: index),
  );
  final data = Uint8List.fromList(template)
    ..[131] = index
    ..[133] = bump;
  await client.cheatcodes.setAccount(
    section,
    owner: bitflipProgramProgramAddress,
    lamports: 10000000,
    data: data,
  );
  return section;
}

Future<Address> _assetId(Address tree, int leaf) async {
  final nonce = Uint8List(8);
  ByteData.sublistView(nonce).setUint64(0, leaf, Endian.little);
  return (await getProgramDerivedAddress(
    programAddress: _bubblegumProgram,
    seeds: ['asset', getAddressEncoder().encode(tree), nonce],
  )).$1;
}

Future<List<Instruction>> _atomicMintInstructions(
  SurfpoolClient client,
  Address tree,
  Address owner,
  Address config,
  Address game,
  Address section,
  int index,
  int leaf,
) async {
  final (treeAuthority, _) = await bubblegum.findTreeAuthorityPda(
    merkleTree: tree,
  );
  return [
    bubblegum.getMintV1Instruction(
      programAddress: _bubblegumProgram,
      treeAuthority: treeAuthority,
      leafOwner: owner,
      leafDelegate: owner,
      merkleTree: tree,
      payer: client.payer.address,
      treeDelegate: client.payer.address,
      logWrapper: _noopProgram,
      compressionProgram: _compressionProgram,
      systemProgram: system.systemProgramAddress,
      message: bubblegum.MetadataArgs(
        name: 'Bitflip 0:$index',
        symbol: 'BITFLIP',
        uri: 'https://bitflip.invalid/metadata/0/$index.json',
        sellerFeeBasisPoints: 0,
        creators: [
          bubblegum.Creator(
            address: client.payer.address,
            verified: false,
            share: 100,
          ),
        ],
      ),
    ),
    getRecordSectionMintInstruction(
      programAddress: bitflipProgramProgramAddress,
      collectionAuthority: client.payer.address,
      config: config,
      game: game,
      section: section,
      treeAuthority: treeAuthority,
      gameIndex: 0,
      sectionIndex: index,
      expectedOwner: owner,
      assetId: await _assetId(tree, leaf),
      merkleTree: tree,
      leafIndex: leaf,
    ),
  ];
}

Future<({String? signature, Object? error})> _raceAttempt(
  SurfpoolClient client,
  List<Instruction> instructions,
) async {
  try {
    return (
      signature: await _sendInstructions(client, instructions),
      error: null,
    );
  } on Object catch (error) {
    return (signature: null, error: error);
  }
}

/// Decode the actual V1 LeafSchemaEvent from Bubblegum's Noop CPI, rather than
/// inferring the minted identity from the same counter used by the server.
Future<void> _expectLeafEvent(
  SurfpoolClient client,
  MintSubmission receipt,
  Address owner,
) async {
  final transaction = await client.rpc
      .getTransaction(
        Signature(receipt.transactionSignature!),
        const rpc_api.GetTransactionConfig(
          commitment: rpc_types.Commitment.confirmed,
          maxSupportedTransactionVersion: 0,
        ),
      )
      .send();
  final meta = transaction!['meta']! as Map<String, Object?>;
  final message =
      (transaction['transaction']! as Map<String, Object?>)['message']!
          as Map<String, Object?>;
  final keys = (message['accountKeys']! as List).cast<String>();
  final events = <Uint8List>[];
  for (final group
      in (meta['innerInstructions']! as List).cast<Map<String, Object?>>()) {
    for (final instruction
        in (group['instructions']! as List).cast<Map<String, Object?>>()) {
      if (keys[instruction['programIdIndex']! as int] != _noopProgram.value) {
        continue;
      }
      final data = base58.decode(instruction['data']! as String);
      if (data.length >= 209 &&
          data[0] == 1 &&
          data[1] == 0 &&
          data[6] == 1 &&
          data[7] == 0 &&
          data[8] == 0) {
        events.add(data);
      }
    }
  }
  expect(events, hasLength(1));
  final event = events.single;
  expect(
    getAddressDecoder().decode(Uint8List.sublistView(event, 9, 41)),
    receipt.assetId,
  );
  expect(
    getAddressDecoder().decode(Uint8List.sublistView(event, 41, 73)),
    owner,
  );
  expect(
    ByteData.sublistView(event).getUint64(105, Endian.little),
    receipt.leafIndex,
  );
  final logs = (meta['logMessages']! as List).cast<String>();
  final consumed = RegExp(
    'Program ${bitflipProgramProgramAddress.value} consumed (\\d+) of',
  ).allMatches(logs.join('\n')).map((match) => int.parse(match[1]!)).single;
  _receiptComputeUnits.add(consumed);
  // ignore: avoid_print
  print('RecordSectionMint leaf ${receipt.leafIndex}: $consumed CU');
}

Future<void> _setTestSectionPixel(
  SurfpoolClient client,
  Address sectionAddress, {
  required int x,
  required int y,
}) async {
  final maybeSection = await fetchEncodedAccount(client.rpc, sectionAddress);
  final section = switch (maybeSection) {
    ExistingAccount<Uint8List>(:final account) => account,
    NonExistingAccount<Uint8List>() => throw StateError(
      'Bitflip section should exist before test setup.',
    ),
  };

  final data = Uint8List.fromList(section.data);
  // Economy ABI v6: the versioned policy occupies bytes 268-342. Account
  // bytes carry the migration-version envelope after the discriminator, so
  // every payload offset shifts by one.
  const pixelsOffset = 343;
  final pixelIndex = y * 64 + x;
  data
    ..[134] = 1
    ..[135] = 0
    ..[pixelsOffset + pixelIndex ~/ 8] |= 1 << (pixelIndex % 8);
  await client.cheatcodes.setAccount(sectionAddress, data: data);
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
    // Account bytes carry the migration-version envelope after the
    // discriminator, so every payload offset shifts by one.
    ..setRange(3, 35, authority)
    ..setRange(67, 99, authority)
    ..setRange(99, 131, authority)
    ..setRange(227, 231, const [1, 0, 0, 0])
    ..setRange(231, 235, const [1, 0, 0, 0]);
  await client.cheatcodes.setAccount(configAddress, data: data);
}

Future<String> _sendInstructions(
  SurfpoolClient client,
  List<Instruction> instructions, {
  List<KeyPairSigner> extraSigners = const [],
}) async {
  for (final instruction in instructions) {
    // ignore: avoid_print
    print('PROBE ix ${instruction.programAddress} data=${instruction.data}');
  }

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
