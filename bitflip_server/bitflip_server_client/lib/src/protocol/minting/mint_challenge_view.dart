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

abstract class MintChallengeView
    implements _isc.SerializableModel, _isc.ProtocolSerialization {
  MintChallengeView._({
    required this.nonce,
    required this.message,
    required this.expiresAt,
  });

  factory MintChallengeView({
    required String nonce,
    required String message,
    required DateTime expiresAt,
  }) = _MintChallengeViewImpl;

  factory MintChallengeView.fromJson(Map<String, dynamic> jsonSerialization) {
    return MintChallengeView(
      nonce: jsonSerialization['nonce'] as String,
      message: jsonSerialization['message'] as String,
      expiresAt: _isc.DateTimeJsonExtension.fromJson(
        jsonSerialization['expiresAt'],
      ),
    );
  }

  String nonce;

  String message;

  DateTime expiresAt;

  /// Returns a shallow copy of this [MintChallengeView]
  /// with some or all fields replaced by the given arguments.
  @_isc.useResult
  MintChallengeView copyWith({
    String? nonce,
    String? message,
    DateTime? expiresAt,
  });
  @override
  Map<String, dynamic> toJson() {
    return {
      '__className__': 'MintChallengeView',
      'nonce': nonce,
      'message': message,
      'expiresAt': expiresAt.toJson(),
    };
  }

  @override
  Map<String, dynamic> toJsonForProtocol() {
    return {
      '__className__': 'MintChallengeView',
      'nonce': nonce,
      'message': message,
      'expiresAt': expiresAt.toJson(),
    };
  }

  @override
  String toString() {
    return _isc.SerializationManager.encode(this);
  }
}

class _MintChallengeViewImpl extends MintChallengeView {
  _MintChallengeViewImpl({
    required String nonce,
    required String message,
    required DateTime expiresAt,
  }) : super._(
         nonce: nonce,
         message: message,
         expiresAt: expiresAt,
       );

  /// Returns a shallow copy of this [MintChallengeView]
  /// with some or all fields replaced by the given arguments.
  @_isc.useResult
  @override
  MintChallengeView copyWith({
    String? nonce,
    String? message,
    DateTime? expiresAt,
  }) {
    return MintChallengeView(
      nonce: nonce ?? this.nonce,
      message: message ?? this.message,
      expiresAt: expiresAt ?? this.expiresAt,
    );
  }
}
