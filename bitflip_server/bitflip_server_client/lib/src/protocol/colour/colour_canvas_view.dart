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
import 'dart:typed_data' as _idt;
import 'package:serverpod_client/serverpod_client.dart' as _isc;

abstract class ColourCanvasView
    implements _isc.SerializableModel, _isc.ProtocolSerialization {
  ColourCanvasView._({
    required this.gameIndex,
    required this.sectionIndex,
    required this.policyVersion,
    required this.highestRevision,
    required this.colours,
  });

  factory ColourCanvasView({
    required int gameIndex,
    required int sectionIndex,
    required int policyVersion,
    required int highestRevision,
    required _idt.ByteData colours,
  }) = _ColourCanvasViewImpl;

  factory ColourCanvasView.fromJson(Map<String, dynamic> jsonSerialization) {
    return ColourCanvasView(
      gameIndex: jsonSerialization['gameIndex'] as int,
      sectionIndex: jsonSerialization['sectionIndex'] as int,
      policyVersion: jsonSerialization['policyVersion'] as int,
      highestRevision: jsonSerialization['highestRevision'] as int,
      colours: _isc.ByteDataJsonExtension.fromJson(
        jsonSerialization['colours'],
      ),
    );
  }

  int gameIndex;

  int sectionIndex;

  int policyVersion;

  int highestRevision;

  _idt.ByteData colours;

  /// Returns a shallow copy of this [ColourCanvasView]
  /// with some or all fields replaced by the given arguments.
  @_isc.useResult
  ColourCanvasView copyWith({
    int? gameIndex,
    int? sectionIndex,
    int? policyVersion,
    int? highestRevision,
    _idt.ByteData? colours,
  });
  @override
  Map<String, dynamic> toJson() {
    return {
      '__className__': 'ColourCanvasView',
      'gameIndex': gameIndex,
      'sectionIndex': sectionIndex,
      'policyVersion': policyVersion,
      'highestRevision': highestRevision,
      'colours': colours.toJson(),
    };
  }

  @override
  Map<String, dynamic> toJsonForProtocol() {
    return {
      '__className__': 'ColourCanvasView',
      'gameIndex': gameIndex,
      'sectionIndex': sectionIndex,
      'policyVersion': policyVersion,
      'highestRevision': highestRevision,
      'colours': colours.toJson(),
    };
  }

  @override
  String toString() {
    return _isc.SerializationManager.encode(this);
  }
}

class _ColourCanvasViewImpl extends ColourCanvasView {
  _ColourCanvasViewImpl({
    required int gameIndex,
    required int sectionIndex,
    required int policyVersion,
    required int highestRevision,
    required _idt.ByteData colours,
  }) : super._(
         gameIndex: gameIndex,
         sectionIndex: sectionIndex,
         policyVersion: policyVersion,
         highestRevision: highestRevision,
         colours: colours,
       );

  /// Returns a shallow copy of this [ColourCanvasView]
  /// with some or all fields replaced by the given arguments.
  @_isc.useResult
  @override
  ColourCanvasView copyWith({
    int? gameIndex,
    int? sectionIndex,
    int? policyVersion,
    int? highestRevision,
    _idt.ByteData? colours,
  }) {
    return ColourCanvasView(
      gameIndex: gameIndex ?? this.gameIndex,
      sectionIndex: sectionIndex ?? this.sectionIndex,
      policyVersion: policyVersion ?? this.policyVersion,
      highestRevision: highestRevision ?? this.highestRevision,
      colours: colours ?? this.colours.clone(),
    );
  }
}
