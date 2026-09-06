/* AUTOMATICALLY GENERATED CODE DO NOT MODIFY */
/*   To generate run: "serverpod generate"    */

// ignore_for_file: implementation_imports
// ignore_for_file: library_private_types_in_public_api
// ignore_for_file: non_constant_identifier_names
// ignore_for_file: public_member_api_docs
// ignore_for_file: type_literal_in_constant_pattern
// ignore_for_file: use_super_parameters
// ignore_for_file: invalid_use_of_internal_member
// ignore_for_file: dead_code, unnecessary_type_check

// ignore_for_file: no_leading_underscores_for_library_prefixes
import 'package:serverpod/protocol.dart' as _isp;
import 'package:serverpod/serverpod.dart' as _is;
import 'colour/colour_canvas_state.dart' as _iexdv2yq;
import 'colour/colour_canvas_view.dart' as _i7up465y;
import 'minting/mint_challenge.dart' as _ilupy5f9;
import 'minting/mint_challenge_view.dart' as _i8azp9qk;
import 'minting/mint_section_result.dart' as _icvzv24k;
export 'colour/colour_canvas_state.dart';
export 'colour/colour_canvas_view.dart';
export 'minting/mint_challenge.dart';
export 'minting/mint_challenge_view.dart';
export 'minting/mint_section_result.dart';

class Protocol extends _is.DatabaseSerializationManager {
  Protocol._();

  factory Protocol() => _instance;

  static final Protocol _instance = Protocol._();

  static List<_isp.TableDefinition> get targetTableDefinitions => [
    _isp.TableDefinition(
      name: 'bitflip_colour_canvas_state',
      dartName: 'ColourCanvasState',
      schema: 'public',
      module: 'bitflip_server',
      columns: [
        _isp.ColumnDefinition(
          name: 'id',
          columnType: _isp.ColumnType.uuid,
          isNullable: false,
          dartType: 'UuidValue?',
          columnDefault: 'random_v7',
        ),
        _isp.ColumnDefinition(
          name: 'gameIndex',
          columnType: _isp.ColumnType.bigint,
          isNullable: false,
          dartType: 'int',
        ),
        _isp.ColumnDefinition(
          name: 'sectionIndex',
          columnType: _isp.ColumnType.bigint,
          isNullable: false,
          dartType: 'int',
        ),
        _isp.ColumnDefinition(
          name: 'policyVersion',
          columnType: _isp.ColumnType.bigint,
          isNullable: false,
          dartType: 'int',
        ),
        _isp.ColumnDefinition(
          name: 'highestRevision',
          columnType: _isp.ColumnType.bigint,
          isNullable: false,
          dartType: 'int',
        ),
        _isp.ColumnDefinition(
          name: 'colours',
          columnType: _isp.ColumnType.bytea,
          isNullable: false,
          dartType: 'dart:typed_data:ByteData',
        ),
        _isp.ColumnDefinition(
          name: 'pixelRevisions',
          columnType: _isp.ColumnType.bytea,
          isNullable: false,
          dartType: 'dart:typed_data:ByteData',
        ),
        _isp.ColumnDefinition(
          name: 'updatedAt',
          columnType: _isp.ColumnType.timestampWithoutTimeZone,
          isNullable: false,
          dartType: 'DateTime',
        ),
      ],
      foreignKeys: [],
      indexes: [
        _isp.IndexDefinition(
          indexName: 'bitflip_colour_canvas_section_idx',
          tableSpace: null,
          elements: [
            _isp.IndexElementDefinition(
              type: _isp.IndexElementDefinitionType.column,
              definition: 'gameIndex',
            ),
            _isp.IndexElementDefinition(
              type: _isp.IndexElementDefinitionType.column,
              definition: 'sectionIndex',
            ),
          ],
          type: 'btree',
          isUnique: true,
          isPrimary: false,
        ),
      ],
      managed: true,
    ),
    _isp.TableDefinition(
      name: 'bitflip_mint_challenge',
      dartName: 'MintChallenge',
      schema: 'public',
      module: 'bitflip_server',
      columns: [
        _isp.ColumnDefinition(
          name: 'id',
          columnType: _isp.ColumnType.uuid,
          isNullable: false,
          dartType: 'UuidValue?',
          columnDefault: 'random_v7',
        ),
        _isp.ColumnDefinition(
          name: 'walletAddress',
          columnType: _isp.ColumnType.text,
          isNullable: false,
          dartType: 'String',
        ),
        _isp.ColumnDefinition(
          name: 'gameIndex',
          columnType: _isp.ColumnType.bigint,
          isNullable: false,
          dartType: 'int',
        ),
        _isp.ColumnDefinition(
          name: 'sectionIndex',
          columnType: _isp.ColumnType.bigint,
          isNullable: false,
          dartType: 'int',
        ),
        _isp.ColumnDefinition(
          name: 'nonce',
          columnType: _isp.ColumnType.text,
          isNullable: false,
          dartType: 'String',
        ),
        _isp.ColumnDefinition(
          name: 'message',
          columnType: _isp.ColumnType.text,
          isNullable: false,
          dartType: 'String',
        ),
        _isp.ColumnDefinition(
          name: 'createdAt',
          columnType: _isp.ColumnType.timestampWithoutTimeZone,
          isNullable: false,
          dartType: 'DateTime',
        ),
        _isp.ColumnDefinition(
          name: 'expiresAt',
          columnType: _isp.ColumnType.timestampWithoutTimeZone,
          isNullable: false,
          dartType: 'DateTime',
        ),
        _isp.ColumnDefinition(
          name: 'usedAt',
          columnType: _isp.ColumnType.timestampWithoutTimeZone,
          isNullable: true,
          dartType: 'DateTime?',
        ),
      ],
      foreignKeys: [],
      indexes: [
        _isp.IndexDefinition(
          indexName: 'bitflip_mint_challenge_nonce_idx',
          tableSpace: null,
          elements: [
            _isp.IndexElementDefinition(
              type: _isp.IndexElementDefinitionType.column,
              definition: 'nonce',
            ),
          ],
          type: 'btree',
          isUnique: true,
          isPrimary: false,
        ),
        _isp.IndexDefinition(
          indexName: 'bitflip_mint_challenge_wallet_created_idx',
          tableSpace: null,
          elements: [
            _isp.IndexElementDefinition(
              type: _isp.IndexElementDefinitionType.column,
              definition: 'walletAddress',
            ),
            _isp.IndexElementDefinition(
              type: _isp.IndexElementDefinitionType.column,
              definition: 'createdAt',
            ),
          ],
          type: 'btree',
          isUnique: false,
          isPrimary: false,
        ),
        _isp.IndexDefinition(
          indexName: 'bitflip_mint_challenge_expiry_idx',
          tableSpace: null,
          elements: [
            _isp.IndexElementDefinition(
              type: _isp.IndexElementDefinitionType.column,
              definition: 'expiresAt',
            ),
          ],
          type: 'btree',
          isUnique: false,
          isPrimary: false,
        ),
      ],
      managed: true,
    ),
    ..._isp.Protocol.targetTableDefinitions,
  ];

  static String? getClassNameFromObjectJson(dynamic data) {
    if (data is! Map) return null;
    final className = data['__className__'] as String?;
    return className;
  }

  @override
  T deserialize<T>(
    dynamic data, [
    Type? t,
  ]) {
    t ??= T;

    final dataClassName = getClassNameFromObjectJson(data);
    if (dataClassName != null && dataClassName != getClassNameForType(t)) {
      try {
        return deserializeByClassName({
          'className': dataClassName,
          'data': data,
        });
      } on _is.DeserializationClassNameNotFoundException catch (_) {
        // If the className is not recognized (e.g., older client receiving
        // data with a new subtype), fall back to deserializing without the
        // className, using the expected type T.
      }
    }

    if (t == _iexdv2yq.ColourCanvasState) {
      return _iexdv2yq.ColourCanvasState.fromJson(data) as T;
    }
    if (t == _i7up465y.ColourCanvasView) {
      return _i7up465y.ColourCanvasView.fromJson(data) as T;
    }
    if (t == _ilupy5f9.MintChallenge) {
      return _ilupy5f9.MintChallenge.fromJson(data) as T;
    }
    if (t == _i8azp9qk.MintChallengeView) {
      return _i8azp9qk.MintChallengeView.fromJson(data) as T;
    }
    if (t == _icvzv24k.MintSectionResult) {
      return _icvzv24k.MintSectionResult.fromJson(data) as T;
    }
    if (t == _is.getType<_iexdv2yq.ColourCanvasState?>()) {
      return (data != null ? _iexdv2yq.ColourCanvasState.fromJson(data) : null)
          as T;
    }
    if (t == _is.getType<_i7up465y.ColourCanvasView?>()) {
      return (data != null ? _i7up465y.ColourCanvasView.fromJson(data) : null)
          as T;
    }
    if (t == _is.getType<_ilupy5f9.MintChallenge?>()) {
      return (data != null ? _ilupy5f9.MintChallenge.fromJson(data) : null)
          as T;
    }
    if (t == _is.getType<_i8azp9qk.MintChallengeView?>()) {
      return (data != null ? _i8azp9qk.MintChallengeView.fromJson(data) : null)
          as T;
    }
    if (t == _is.getType<_icvzv24k.MintSectionResult?>()) {
      return (data != null ? _icvzv24k.MintSectionResult.fromJson(data) : null)
          as T;
    }
    try {
      return _isp.Protocol().deserialize<T>(data, t);
    } on _is.DeserializationTypeNotFoundException catch (_) {}
    return super.deserialize<T>(data, t);
  }

  static String? getClassNameForType(Type type) {
    return switch (type) {
      _iexdv2yq.ColourCanvasState => 'ColourCanvasState',
      _i7up465y.ColourCanvasView => 'ColourCanvasView',
      _ilupy5f9.MintChallenge => 'MintChallenge',
      _i8azp9qk.MintChallengeView => 'MintChallengeView',
      _icvzv24k.MintSectionResult => 'MintSectionResult',
      _ => null,
    };
  }

  @override
  String? getClassNameForObject(Object? data) {
    String? className = super.getClassNameForObject(data);
    if (className != null) return className;

    if (data is Map<String, dynamic> && data['__className__'] is String) {
      return (data['__className__'] as String).replaceFirst(
        'bitflip_server.',
        '',
      );
    }

    switch (data) {
      case _iexdv2yq.ColourCanvasState():
        return 'ColourCanvasState';
      case _i7up465y.ColourCanvasView():
        return 'ColourCanvasView';
      case _ilupy5f9.MintChallenge():
        return 'MintChallenge';
      case _i8azp9qk.MintChallengeView():
        return 'MintChallengeView';
      case _icvzv24k.MintSectionResult():
        return 'MintSectionResult';
    }
    className = _isp.Protocol().getClassNameForObject(data);
    if (className != null) {
      return className.contains('.') ? className : 'serverpod.$className';
    }
    return null;
  }

  @override
  dynamic deserializeByClassName(Map<String, dynamic> data) {
    var dataClassName = data['className'];
    if (dataClassName is! String) {
      return super.deserializeByClassName(data);
    }
    if (dataClassName == 'ColourCanvasState') {
      return deserialize<_iexdv2yq.ColourCanvasState>(data['data']);
    }
    if (dataClassName == 'ColourCanvasView') {
      return deserialize<_i7up465y.ColourCanvasView>(data['data']);
    }
    if (dataClassName == 'MintChallenge') {
      return deserialize<_ilupy5f9.MintChallenge>(data['data']);
    }
    if (dataClassName == 'MintChallengeView') {
      return deserialize<_i8azp9qk.MintChallengeView>(data['data']);
    }
    if (dataClassName == 'MintSectionResult') {
      return deserialize<_icvzv24k.MintSectionResult>(data['data']);
    }
    if (dataClassName.startsWith('serverpod.')) {
      data['className'] = dataClassName.substring(10);
      return _isp.Protocol().deserializeByClassName(data);
    }
    return super.deserializeByClassName(data);
  }

  @override
  _is.Table? getTableForType(Type t) {
    {
      var table = _isp.Protocol().getTableForType(t);
      if (table != null) {
        return table;
      }
    }
    switch (t) {
      case _iexdv2yq.ColourCanvasState:
        return _iexdv2yq.ColourCanvasState.t;
      case _ilupy5f9.MintChallenge:
        return _ilupy5f9.MintChallenge.t;
    }
    return null;
  }

  @override
  List<_isp.TableDefinition> getTargetTableDefinitions() =>
      targetTableDefinitions;

  @override
  String getModuleName() => 'bitflip_server';

  /// Maps any `Record`s known to this [Protocol] to their JSON representation
  ///
  /// Throws in case the record type is not known.
  ///
  /// This method will return `null` (only) for `null` inputs.
  Map<String, dynamic>? mapRecordToJson(Record? record) {
    if (record == null) {
      return null;
    }
    try {
      return _isp.Protocol().mapRecordToJson(record);
    } catch (_) {}
    throw Exception('Unsupported record type ${record.runtimeType}');
  }
}
