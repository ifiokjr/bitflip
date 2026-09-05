/* AUTOMATICALLY GENERATED CODE DO NOT MODIFY */
/*   To generate run: "serverpod generate"    */

// ignore_for_file: implementation_imports
// ignore_for_file: library_private_types_in_public_api
// ignore_for_file: non_constant_identifier_names
// ignore_for_file: public_member_api_docs
// ignore_for_file: type_literal_in_constant_pattern
// ignore_for_file: use_super_parameters
// ignore_for_file: invalid_use_of_internal_member

// ignore_for_file: no_leading_underscores_for_library_prefixes
import 'package:serverpod_client/serverpod_client.dart' as _isc;

abstract class MintSectionResult
    implements _isc.SerializableModel, _isc.ProtocolSerialization {
  MintSectionResult._({
    required this.assetId,
    required this.merkleTree,
    required this.leafIndex,
    this.transactionSignature,
    required this.alreadyMinted,
  });

  factory MintSectionResult({
    required String assetId,
    required String merkleTree,
    required int leafIndex,
    String? transactionSignature,
    required bool alreadyMinted,
  }) = _MintSectionResultImpl;

  factory MintSectionResult.fromJson(Map<String, dynamic> jsonSerialization) {
    return MintSectionResult(
      assetId: jsonSerialization['assetId'] as String,
      merkleTree: jsonSerialization['merkleTree'] as String,
      leafIndex: jsonSerialization['leafIndex'] as int,
      transactionSignature:
          jsonSerialization['transactionSignature'] as String?,
      alreadyMinted: _isc.BoolJsonExtension.fromJson(
        jsonSerialization['alreadyMinted'],
      ),
    );
  }

  String assetId;

  String merkleTree;

  int leafIndex;

  String? transactionSignature;

  bool alreadyMinted;

  /// Returns a shallow copy of this [MintSectionResult]
  /// with some or all fields replaced by the given arguments.
  @_isc.useResult
  MintSectionResult copyWith({
    String? assetId,
    String? merkleTree,
    int? leafIndex,
    String? transactionSignature,
    bool? alreadyMinted,
  });
  @override
  Map<String, dynamic> toJson() {
    return {
      '__className__': 'MintSectionResult',
      'assetId': assetId,
      'merkleTree': merkleTree,
      'leafIndex': leafIndex,
      if (transactionSignature != null)
        'transactionSignature': transactionSignature,
      'alreadyMinted': alreadyMinted,
    };
  }

  @override
  Map<String, dynamic> toJsonForProtocol() {
    return {
      '__className__': 'MintSectionResult',
      'assetId': assetId,
      'merkleTree': merkleTree,
      'leafIndex': leafIndex,
      if (transactionSignature != null)
        'transactionSignature': transactionSignature,
      'alreadyMinted': alreadyMinted,
    };
  }

  @override
  String toString() {
    return _isc.SerializationManager.encode(this);
  }
}

class _Undefined {}

class _MintSectionResultImpl extends MintSectionResult {
  _MintSectionResultImpl({
    required String assetId,
    required String merkleTree,
    required int leafIndex,
    String? transactionSignature,
    required bool alreadyMinted,
  }) : super._(
         assetId: assetId,
         merkleTree: merkleTree,
         leafIndex: leafIndex,
         transactionSignature: transactionSignature,
         alreadyMinted: alreadyMinted,
       );

  /// Returns a shallow copy of this [MintSectionResult]
  /// with some or all fields replaced by the given arguments.
  @_isc.useResult
  @override
  MintSectionResult copyWith({
    String? assetId,
    String? merkleTree,
    int? leafIndex,
    Object? transactionSignature = _Undefined,
    bool? alreadyMinted,
  }) {
    return MintSectionResult(
      assetId: assetId ?? this.assetId,
      merkleTree: merkleTree ?? this.merkleTree,
      leafIndex: leafIndex ?? this.leafIndex,
      transactionSignature: transactionSignature is String?
          ? transactionSignature
          : this.transactionSignature,
      alreadyMinted: alreadyMinted ?? this.alreadyMinted,
    );
  }
}
