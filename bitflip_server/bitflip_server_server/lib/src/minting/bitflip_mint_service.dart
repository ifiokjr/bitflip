import 'dart:convert';
import 'dart:async';
import 'dart:typed_data';

import 'package:bitflip_program/bitflip_program.dart';
import 'package:solana_kit/solana_kit.dart';
import 'package:solana_kit_mpl_bubblegum/solana_kit_mpl_bubblegum.dart'
    as bubblegum;
import 'package:solana_kit_rpc_api/solana_kit_rpc_api.dart' as rpc_api;
import 'package:solana_kit_rpc_spec/solana_kit_rpc_spec.dart' as rpc_spec;
import 'package:solana_kit_rpc_types/solana_kit_rpc_types.dart' as rpc_types;
import 'package:solana_kit_system/solana_kit_system.dart' as system;

const _computeBudgetProgram = Address(
  'ComputeBudget111111111111111111111111111111',
);
const _accountCompressionProgram = Address(
  'cmtDvXumGCrqC1Age74AVPhSRVXJMd8PJS91L8KbNCK',
);
const _noopProgram = Address('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV');

final class MintableSection {
  const MintableSection({
    required this.gameIndex,
    required this.sectionIndex,
    required this.owner,
    required this.status,
    required this.collectionAuthority,
    required this.configAddress,
    required this.gameAddress,
    required this.sectionAddress,
    required this.bitmap,
    this.assetId,
    this.merkleTree,
    this.leafIndex,
  });

  final int gameIndex;
  final int sectionIndex;
  final Address owner;
  final int status;
  final Address collectionAuthority;
  final Address configAddress;
  final Address gameAddress;
  final Address sectionAddress;
  final Uint8List bitmap;
  final Address? assetId;
  final Address? merkleTree;
  final int? leafIndex;

  bool get isSealed => status == 2;
  bool get isMinted => status == 3;
}

final class MintSubmission {
  const MintSubmission({
    required this.assetId,
    required this.merkleTree,
    required this.leafIndex,
    required this.alreadyMinted,
    this.transactionSignature,
  });

  final Address assetId;
  final Address merkleTree;
  final int leafIndex;
  final String? transactionSignature;
  final bool alreadyMinted;
}

abstract interface class BitflipMintService {
  Future<MintableSection> loadSection(int gameIndex, int sectionIndex);

  Future<MintSubmission> mint(MintableSection section);
}

final class SolanaBitflipMintService implements BitflipMintService {
  SolanaBitflipMintService({
    required this.rpc,
    required this.merkleTreeAddress,
    required this.operatorPrivateKey,
    required String metadataBaseUrl,
    this.gameIndex,
    this.priorityFeeMicroLamports = 0,
    this.rpcTimeout = const Duration(seconds: 12),
    this.confirmationTimeout = const Duration(seconds: 60),
    this.maximumSubmitAttempts = 3,
  }) : metadataBaseUrl = _validatedMetadataBaseUrl(metadataBaseUrl);

  factory SolanaBitflipMintService.fromEnvironment(
    Map<String, String> environment, {
    required bool production,
    bool requireExplicitConfiguration = false,
  }) {
    final strictConfiguration = production || requireExplicitConfiguration;
    final rpcUrl = _environmentValue(
      environment,
      'SOLANA_RPC_URL',
      requiredForRelease: strictConfiguration,
      developmentDefault: 'http://127.0.0.1:8899',
    );
    final metadataBaseUrl = _environmentValue(
      environment,
      'BITFLIP_METADATA_BASE_URL',
      requiredForRelease: strictConfiguration,
      developmentDefault: 'http://localhost:8082',
    );
    final gameIndexValue = _environmentValue(
      environment,
      'BITFLIP_GAME_INDEX',
      requiredForRelease: strictConfiguration,
      developmentDefault: '0',
    );
    final gameIndex = int.tryParse(gameIndexValue);
    if (gameIndex == null || gameIndex < 0 || gameIndex > 255) {
      throw StateError('BITFLIP_GAME_INDEX must be between 0 and 255.');
    }
    final tree = environment['BITFLIP_MERKLE_TREE']?.trim();
    final operator = environment['BITFLIP_OPERATOR_PRIVATE_KEY']?.trim();
    final priorityFeeMicroLamports = _environmentInteger(
      environment,
      'BITFLIP_PRIORITY_FEE_MICROLAMPORTS',
      requiredForRelease: strictConfiguration,
      developmentDefault: 0,
      minimum: 0,
      maximum: 10000000,
    );
    final rpcTimeoutSeconds = _optionalEnvironmentInteger(
      environment,
      'BITFLIP_RPC_TIMEOUT_SECONDS',
      defaultValue: 12,
      minimum: 2,
      maximum: 60,
    );
    final maximumSubmitAttempts = _optionalEnvironmentInteger(
      environment,
      'BITFLIP_MINT_MAX_ATTEMPTS',
      defaultValue: 3,
      minimum: 1,
      maximum: 5,
    );
    if (strictConfiguration) {
      final cluster = _requiredEnvironmentValue(environment, 'BITFLIP_CLUSTER');
      if (production && cluster != 'mainnet') {
        throw StateError('Production BITFLIP_CLUSTER must be mainnet.');
      }
      if (!production && cluster != 'devnet') {
        throw StateError('Staging BITFLIP_CLUSTER must be devnet.');
      }
      _requirePublicHttps('SOLANA_RPC_URL', rpcUrl);
      _requirePublicHttps('BITFLIP_METADATA_BASE_URL', metadataBaseUrl);
      if (production &&
          (rpcUrl.toLowerCase().contains('devnet') ||
              rpcUrl.toLowerCase().contains('testnet'))) {
        throw StateError('Production SOLANA_RPC_URL must target mainnet.');
      }
      _requiredConfiguration(tree, 'BITFLIP_MERKLE_TREE');
      _requiredConfiguration(operator, 'BITFLIP_OPERATOR_PRIVATE_KEY');
    }
    final service = SolanaBitflipMintService(
      rpc: createSolanaRpc(url: rpcUrl, allowInsecureHttp: _isLoopback(rpcUrl)),
      merkleTreeAddress: tree,
      operatorPrivateKey: operator,
      metadataBaseUrl: metadataBaseUrl,
      gameIndex: gameIndex,
      priorityFeeMicroLamports: priorityFeeMicroLamports,
      rpcTimeout: Duration(seconds: rpcTimeoutSeconds),
      maximumSubmitAttempts: maximumSubmitAttempts,
    );
    if (strictConfiguration) service.validateSigningConfiguration();
    return service;
  }

  final rpc_spec.Rpc rpc;
  final String? merkleTreeAddress;
  final String? operatorPrivateKey;
  final String metadataBaseUrl;
  final int? gameIndex;
  final int priorityFeeMicroLamports;
  final Duration rpcTimeout;
  final Duration confirmationTimeout;
  final int maximumSubmitAttempts;

  void validateSigningConfiguration() {
    getAddressEncoder().encode(
      Address(_requiredConfiguration(merkleTreeAddress, 'BITFLIP_MERKLE_TREE')),
    );
    final signer = _operatorSigner();
    signer.keyPair.dispose();
  }

  @override
  Future<MintableSection> loadSection(int gameIndex, int sectionIndex) async {
    _validateIndices(gameIndex, sectionIndex);
    if (this.gameIndex case final configuredGameIndex?
        when configuredGameIndex != gameIndex) {
      throw StateError(
        'Requested game $gameIndex does not match configured game '
        '$configuredGameIndex.',
      );
    }
    final (configAddress, _) = await findConfigPda(
      programAddress: bitflipProgramProgramAddress,
    );
    final (gameAddress, _) = await findGamePda(
      programAddress: bitflipProgramProgramAddress,
      seeds: GameSeeds(gameIndex: gameIndex),
    );
    final (sectionAddress, _) = await findSectionPda(
      programAddress: bitflipProgramProgramAddress,
      seeds: SectionSeeds(gameIndex: gameIndex, sectionIndex: sectionIndex),
    );
    final accounts = await fetchEncodedAccounts(rpc, [
      configAddress,
      gameAddress,
      sectionAddress,
    ]).timeout(rpcTimeout);
    final configAccount = _requiredProgramAccount(accounts[0], 'config');
    _requiredProgramAccount(accounts[1], 'game');
    final sectionAccount = _requiredProgramAccount(accounts[2], 'section');
    final config = decodeConfigState(configAccount).data;
    final section = decodeSectionState(sectionAccount).data;
    if (section.gameIndex != gameIndex ||
        section.sectionIndex != sectionIndex) {
      throw StateError('The section PDA contains mismatched coordinates.');
    }
    return MintableSection(
      gameIndex: gameIndex,
      sectionIndex: sectionIndex,
      owner: section.owner,
      status: section.status,
      collectionAuthority: config.collectionAuthority,
      configAddress: configAddress,
      gameAddress: gameAddress,
      sectionAddress: sectionAddress,
      bitmap: Uint8List.fromList(section.pixels),
      assetId: section.status == 3 ? section.assetId : null,
      merkleTree: section.status == 3 ? section.merkleTree : null,
      leafIndex: section.status == 3 ? section.leafIndex : null,
    );
  }

  @override
  Future<MintSubmission> mint(MintableSection section) async {
    Object? lastError;
    StackTrace? lastStackTrace;
    for (var attempt = 1; attempt <= maximumSubmitAttempts; attempt++) {
      try {
        return await _mintOnce(section);
      } on Object catch (error, stackTrace) {
        lastError = error;
        lastStackTrace = stackTrace;
        if (attempt == maximumSubmitAttempts) break;
        await Future<void>.delayed(Duration(milliseconds: 250 * attempt));
      }
    }
    Error.throwWithStackTrace(lastError!, lastStackTrace!);
  }

  Future<MintSubmission> _mintOnce(MintableSection section) async {
    final fresh = await loadSection(section.gameIndex, section.sectionIndex);
    if (fresh.isMinted) {
      return MintSubmission(
        assetId: fresh.assetId!,
        merkleTree: fresh.merkleTree!,
        leafIndex: fresh.leafIndex!,
        alreadyMinted: true,
      );
    }
    if (!fresh.isSealed) {
      throw StateError('Only sealed artwork can be minted.');
    }
    final tree = Address(
      _requiredConfiguration(merkleTreeAddress, 'BITFLIP_MERKLE_TREE'),
    );
    final signer = _operatorSigner();
    try {
      if (signer.address != fresh.collectionAuthority) {
        throw StateError(
          'The operator signer does not match the on-chain collection authority.',
        );
      }
      final (treeAuthority, _) = await bubblegum.findTreeAuthorityPda(
        merkleTree: tree,
      );
      final treeConfigAccount = await fetchEncodedAccount(
        rpc,
        treeAuthority,
      ).timeout(rpcTimeout);
      final encodedTreeConfig = switch (treeConfigAccount) {
        ExistingAccount<Uint8List>(:final account) => account,
        NonExistingAccount<Uint8List>() => throw StateError(
          'The configured Bubblegum tree does not exist.',
        ),
      };
      if (encodedTreeConfig.programAddress !=
          bubblegum.mplBubblegumProgramAddressObject) {
        throw StateError(
          'The configured tree authority is not owned by Bubblegum.',
        );
      }
      final treeConfig = _decodeTreeConfig(encodedTreeConfig.data);
      if (treeConfig.isPublic || treeConfig.treeDelegate != signer.address) {
        throw StateError(
          'Bitflip requires a private Bubblegum tree delegated to the operator.',
        );
      }
      final leafIndex = treeConfig.numMinted;
      if (leafIndex < 0 || leafIndex > 0xffffffff) {
        throw StateError('The Bubblegum leaf index exceeds Bitflip capacity.');
      }
      final assetId = await _deriveAssetId(tree, leafIndex);
      final mintInstruction = bubblegum.getMintV1Instruction(
        programAddress: bubblegum.mplBubblegumProgramAddressObject,
        treeAuthority: treeAuthority,
        leafOwner: fresh.owner,
        leafDelegate: fresh.owner,
        merkleTree: tree,
        payer: signer.address,
        treeDelegate: signer.address,
        logWrapper: _noopProgram,
        compressionProgram: _accountCompressionProgram,
        systemProgram: system.systemProgramAddress,
        message: bubblegum.MetadataArgs(
          name:
              'Bitflip ${fresh.gameIndex}:${fresh.sectionIndex.toString().padLeft(3, '0')}',
          symbol: 'BITFLIP',
          uri: _metadataUri(fresh.gameIndex, fresh.sectionIndex),
          sellerFeeBasisPoints: 0,
          creators: [
            bubblegum.Creator(
              address: signer.address,
              verified: false,
              share: 100,
            ),
          ],
        ),
      );
      final recordInstruction = getRecordSectionMintInstruction(
        programAddress: bitflipProgramProgramAddress,
        collectionAuthority: signer.address,
        config: fresh.configAddress,
        game: fresh.gameAddress,
        section: fresh.sectionAddress,
        gameIndex: fresh.gameIndex,
        sectionIndex: fresh.sectionIndex,
        assetId: assetId,
        merkleTree: tree,
        leafIndex: leafIndex,
      );
      final transactionSignature = await _submit(signer, [
        mintInstruction,
        recordInstruction,
      ]);
      return MintSubmission(
        assetId: assetId,
        merkleTree: tree,
        leafIndex: leafIndex,
        transactionSignature: transactionSignature,
        alreadyMinted: false,
      );
    } finally {
      signer.keyPair.dispose();
    }
  }

  Future<String> _submit(
    KeyPairSigner signer,
    List<Instruction> instructions,
  ) async {
    final latest = await rpc.getLatestBlockhashValue().send().timeout(
      rpcTimeout,
    );
    final transaction = compileTransaction(
      createTransactionMessage(version: TransactionVersion.v0)
          .withFeePayer(signer.address)
          .withBlockhashLifetime(
            BlockhashLifetimeConstraint(
              blockhash: latest.value.blockhash.value,
              lastValidBlockHeight: latest.value.lastValidBlockHeight,
            ),
          )
          .appendInstructions([
            _computeUnitLimitInstruction(1400000),
            if (priorityFeeMicroLamports > 0)
              _computeUnitPriceInstruction(priorityFeeMicroLamports),
            ...instructions,
          ]),
    );
    final signed = await signTransactionWithSigners([signer], transaction);
    final transactionSignature = signature(
      await rpc
          .sendTransaction(
            getBase64EncodedWireTransaction(signed),
            const rpc_api.SendTransactionConfig(
              encoding: rpc_types.WireTransactionEncoding.base64,
              preflightCommitment: rpc_types.Commitment.confirmed,
            ),
          )
          .send()
          .timeout(rpcTimeout),
    );
    await waitForTransactionConfirmation(
      rpc: rpc,
      signature: transactionSignature,
      transaction: signed,
      config: const RpcTransactionConfirmationConfig(
        commitment: rpc_types.Commitment.confirmed,
        pollInterval: Duration(milliseconds: 500),
      ),
    ).timeout(confirmationTimeout);
    return transactionSignature.value;
  }

  KeyPairSigner _operatorSigner() {
    final encoded = _requiredConfiguration(
      operatorPrivateKey,
      'BITFLIP_OPERATOR_PRIVATE_KEY',
    );
    final decoded = jsonDecode(encoded);
    if (decoded is! List || decoded.any((value) => value is! int)) {
      throw StateError(
        'BITFLIP_OPERATOR_PRIVATE_KEY must be a JSON byte array.',
      );
    }
    final bytes = Uint8List.fromList(decoded.cast<int>());
    if (bytes.length != 32 && bytes.length != 64) {
      throw StateError('The operator private key must contain 32 or 64 bytes.');
    }
    try {
      return bytes.length == 64
          ? createKeyPairSignerFromBytes(bytes)
          : createKeyPairSignerFromPrivateKeyBytes(bytes);
    } finally {
      bytes.fillRange(0, bytes.length, 0);
    }
  }

  String _metadataUri(int gameIndex, int sectionIndex) {
    final base = metadataBaseUrl.replaceFirst(RegExp(r'/+$'), '');
    return '$base/metadata/$gameIndex/$sectionIndex.json';
  }

  static Future<Address> _deriveAssetId(Address tree, int leafIndex) async {
    final (assetId, _) = await getProgramDerivedAddress(
      programAddress: bubblegum.mplBubblegumProgramAddressObject,
      seeds: ['asset', getAddressEncoder().encode(tree), _u64(leafIndex)],
    );
    return assetId;
  }

  static EncodedAccount _requiredProgramAccount(
    MaybeEncodedAccount account,
    String label,
  ) {
    final encoded = switch (account) {
      ExistingAccount<Uint8List>(:final account) => account,
      NonExistingAccount<Uint8List>() => throw StateError(
        'The Bitflip $label account does not exist.',
      ),
    };
    if (encoded.programAddress != bitflipProgramProgramAddress) {
      throw StateError('The Bitflip $label account has an invalid owner.');
    }
    return encoded;
  }
}

Instruction _computeUnitLimitInstruction(int units) {
  final data = Uint8List(5)..[0] = 2;
  ByteData.sublistView(data).setUint32(1, units, Endian.little);
  return Instruction(
    programAddress: _computeBudgetProgram,
    accounts: const [],
    data: data,
  );
}

Instruction _computeUnitPriceInstruction(int microLamports) {
  final data = Uint8List(9)..[0] = 3;
  ByteData.sublistView(data).setUint64(1, microLamports, Endian.little);
  return Instruction(
    programAddress: _computeBudgetProgram,
    accounts: const [],
    data: data,
  );
}

Uint8List _u64(int value) {
  final bytes = Uint8List(8);
  ByteData.sublistView(bytes).setUint64(0, value, Endian.little);
  return bytes;
}

({Address treeDelegate, int numMinted, bool isPublic}) _decodeTreeConfig(
  Uint8List data,
) {
  const discriminatorSize = 8;
  const addressSize = 32;
  const u64Size = 8;
  const requiredSize =
      discriminatorSize + (addressSize * 2) + (u64Size * 2) + 1;
  if (data.length < requiredSize) {
    throw StateError(
      'The Bubblegum tree config is truncated: expected at least '
      '$requiredSize bytes, received ${data.length}.',
    );
  }

  final delegateOffset = discriminatorSize + addressSize;
  final numMintedOffset = delegateOffset + addressSize + u64Size;
  final publicOffset = numMintedOffset + u64Size;
  return (
    treeDelegate: getAddressDecoder().decode(
      Uint8List.sublistView(data, delegateOffset, delegateOffset + addressSize),
    ),
    numMinted: ByteData.sublistView(
      data,
      numMintedOffset,
      numMintedOffset + u64Size,
    ).getUint64(0, Endian.little),
    isPublic: data[publicOffset] != 0,
  );
}

String _requiredConfiguration(String? value, String name) {
  if (value == null || value.isEmpty) {
    throw StateError('$name is required before compressed NFTs can be minted.');
  }
  return value;
}

String _environmentValue(
  Map<String, String> environment,
  String name, {
  required bool requiredForRelease,
  required String developmentDefault,
}) {
  final value = environment[name]?.trim();
  if (value != null && value.isNotEmpty) return value;
  if (requiredForRelease) return _requiredEnvironmentValue(environment, name);
  return developmentDefault;
}

String _requiredEnvironmentValue(Map<String, String> environment, String name) {
  final value = environment[name]?.trim();
  if (value == null || value.isEmpty) {
    throw StateError('$name is required for release deployments.');
  }
  return value;
}

int _environmentInteger(
  Map<String, String> environment,
  String name, {
  required bool requiredForRelease,
  required int developmentDefault,
  required int minimum,
  required int maximum,
}) {
  final value = environment[name]?.trim();
  if ((value == null || value.isEmpty) && requiredForRelease) {
    throw StateError('$name is required for release deployments.');
  }
  return _validatedInteger(
    name,
    value,
    defaultValue: developmentDefault,
    minimum: minimum,
    maximum: maximum,
  );
}

int _optionalEnvironmentInteger(
  Map<String, String> environment,
  String name, {
  required int defaultValue,
  required int minimum,
  required int maximum,
}) {
  return _validatedInteger(
    name,
    environment[name]?.trim(),
    defaultValue: defaultValue,
    minimum: minimum,
    maximum: maximum,
  );
}

int _validatedInteger(
  String name,
  String? value, {
  required int defaultValue,
  required int minimum,
  required int maximum,
}) {
  final parsed = value == null || value.isEmpty
      ? defaultValue
      : int.tryParse(value);
  if (parsed == null || parsed < minimum || parsed > maximum) {
    throw StateError('$name must be between $minimum and $maximum.');
  }
  return parsed;
}

void _requirePublicHttps(String name, String value) {
  final uri = Uri.tryParse(value);
  final host = uri?.host.toLowerCase();
  if (uri == null ||
      uri.scheme != 'https' ||
      !uri.hasAuthority ||
      host == 'localhost' ||
      host == '127.0.0.1' ||
      host == '::1') {
    throw StateError('$name must use a public HTTPS endpoint in releases.');
  }
}

void _validateIndices(int gameIndex, int sectionIndex) {
  if (gameIndex < 0 || gameIndex > 255) {
    throw RangeError.range(gameIndex, 0, 255, 'gameIndex');
  }
  if (sectionIndex < 0 || sectionIndex > 255) {
    throw RangeError.range(sectionIndex, 0, 255, 'sectionIndex');
  }
}

bool _isLoopback(String value) {
  final host = Uri.tryParse(value)?.host.toLowerCase();
  return host == '127.0.0.1' || host == 'localhost' || host == '::1';
}

String _validatedMetadataBaseUrl(String value) {
  final normalized = value.trim();
  final uri = Uri.tryParse(normalized);
  final validUri =
      uri != null &&
      uri.host.isNotEmpty &&
      uri.userInfo.isEmpty &&
      !uri.hasQuery &&
      !uri.hasFragment;
  final validTransport =
      uri?.scheme == 'https' ||
      (uri?.scheme == 'http' && _isLoopback(normalized));
  if (!validUri || !validTransport) {
    throw StateError(
      'BITFLIP_METADATA_BASE_URL must be an HTTPS origin or an HTTP '
      'loopback URL without credentials, a query, or a fragment.',
    );
  }
  return normalized.replaceFirst(RegExp(r'/+$'), '');
}

abstract final class MintServiceRegistry {
  static BitflipMintService? _service;

  static BitflipMintService get service =>
      _service ??
      (throw StateError('The Bitflip mint service is unavailable.'));

  static void configure(BitflipMintService service) {
    _service = service;
  }
}
