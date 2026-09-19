// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';
import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_core/solana_kit_codecs_core.dart';
import 'package:solana_kit_codecs_data_structures/solana_kit_codecs_data_structures.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';

import 'event_log.dart';

/// Event record `ColourPixelsFlippedEvent`.
class ColourPixelsFlippedEventEvent {
	const ColourPixelsFlippedEventEvent({
		required this.discriminator,
		required this.migrationVersion,
		required this.player,
		required this.policyVersion,
		required this.revision,
		required this.coordinates,
		required this.gameIndex,
		required this.sectionIndex,
		required this.count,
		required this.colour,
	});

	final int discriminator;
	final int migrationVersion;
	final Address player;
	final BigInt policyVersion;
	final BigInt revision;
	final Uint8List coordinates;
	final int gameIndex;
	final int sectionIndex;
	final int count;
	final int colour;

	String get name => 'colourPixelsFlippedEvent';

	String toString() => 'ColourPixelsFlippedEventEvent(discriminator: ${discriminator}, migrationVersion: ${migrationVersion}, player: ${player}, policyVersion: ${policyVersion}, revision: ${revision}, coordinates: ${coordinates}, gameIndex: ${gameIndex}, sectionIndex: ${sectionIndex}, count: ${count}, colour: ${colour})';
}

/// The discriminator this event is emitted under.
const colourPixelsFlippedEventEventDiscriminator = 1;

/// The discriminator bytes as stored at offset zero.
const List<int> _colourPixelsFlippedEventEventDiscriminatorBytes = [1];

/// The version this client was generated from.
const colourPixelsFlippedEventEventMigrationVersion = 0;

/// Exact current byte length of a `ColourPixelsFlippedEvent` record, envelope included.
const colourPixelsFlippedEventEventSize = 86;

/// Decode one current-version `ColourPixelsFlippedEvent` record.
ColourPixelsFlippedEventEvent decodeColourPixelsFlippedEventEvent(Uint8List data) {
	if (data.length != colourPixelsFlippedEventEventSize) {
		throw RangeError('expected exactly ${colourPixelsFlippedEventEventSize} bytes, received ${data.length}');
	}
	var cursor = 0;
	final (v0, c0) = getU8Decoder().read(data, cursor);
	cursor = c0;
	if (v0 != 1) {
		throw RangeError('the provided bytes do not match the "ColourPixelsFlippedEvent" event discriminator');
	}
	final (v1, c1) = getU8Decoder().read(data, cursor);
	cursor = c1;
	if (v1 != 0) {
		throw RangeError(
			v1 < 0
				? 'event migration version mismatch: expected 0, received $v1 (the log predates this client; project it through the checked-in event history or decode it with a client generated from the schema that wrote it)'
				: 'event migration version mismatch: expected 0, received $v1 (the log was written by a newer program; upgrade this client)',
		);
	}
	final (v2, c2) = getAddressDecoder().read(data, cursor);
	cursor = c2;
	final (v3, c3) = getU64Decoder().read(data, cursor);
	cursor = c3;
	final (v4, c4) = getU64Decoder().read(data, cursor);
	cursor = c4;
	final (v5, c5) = fixDecoderSize(getBytesDecoder(), 32).read(data, cursor);
	cursor = c5;
	final (v6, c6) = getU8Decoder().read(data, cursor);
	cursor = c6;
	final (v7, c7) = getU8Decoder().read(data, cursor);
	cursor = c7;
	final (v8, c8) = getU8Decoder().read(data, cursor);
	cursor = c8;
	final (v9, c9) = getU8Decoder().read(data, cursor);
	cursor = c9;

	return ColourPixelsFlippedEventEvent(discriminator: v0, migrationVersion: v1, player: v2, policyVersion: v3, revision: v4, coordinates: v5, gameIndex: v6, sectionIndex: v7, count: v8, colour: v9);
}

/// Event bytes projected into the current shape.
class NormalizedColourPixelsFlippedEventEvent extends BitflipProgramEvent {
	const NormalizedColourPixelsFlippedEventEvent({
		required this.data,
		required this.sourceVersion,
		required this.wasMigrated,
	});

	final ColourPixelsFlippedEventEvent data;

	/// The version carried by the immutable log record, matching the runtime's
	/// `CurrentEventData::source_version`.
	final int sourceVersion;

	/// Whether a historical projection ran.
	final bool wasMigrated;

	@override
	String get name => 'colourPixelsFlippedEvent';
}

/// Adjacent projections from the checked-in migration manifest: `(from, to,
/// automatic, source payload size, destination payload size, moves)`.
const List<(int, int, bool, int, int, List<(int, int, int)>)> _colourPixelsFlippedEventProjectionSteps = [

];

/// Project current or historical bytes into the current shape, mirroring the
/// runtime's `normalize_event_data`. Unknown, future, non-exact, and manual
/// transitions fail closed.
NormalizedColourPixelsFlippedEventEvent normalizeColourPixelsFlippedEventEvent(Uint8List data) {
	if (data.length < 2) {
		throw RangeError('the provided data is too short for the "ColourPixelsFlippedEvent" event envelope');
	}
	for (var index = 0; index < 1; index++) {
		if (data[index] != _colourPixelsFlippedEventEventDiscriminatorBytes[index]) {
			throw RangeError('the provided data does not match the "ColourPixelsFlippedEvent" event discriminator');
		}
	}
	final sourceVersion = data[1];
	if (sourceVersion > 0) {
		throw RangeError(
			'event migration version mismatch: expected 0, received $sourceVersion (the log was written by a newer program; upgrade this client)',
		);
	}
	if (sourceVersion == 0) {
		return NormalizedColourPixelsFlippedEventEvent(
			data: decodeColourPixelsFlippedEventEvent(data),
			sourceVersion: sourceVersion,
			wasMigrated: false,
		);
	}
	final projected = _projectColourPixelsFlippedEventEvent(data, sourceVersion);
	return NormalizedColourPixelsFlippedEventEvent(
		data: decodeColourPixelsFlippedEventEvent(projected),
		sourceVersion: sourceVersion,
		wasMigrated: true,
	);
}

Uint8List _projectColourPixelsFlippedEventEvent(Uint8List data, int sourceVersion) {
	var version = sourceVersion;
	var payload = Uint8List.fromList(data.sublist(2));
	while (version != 0) {
		(int, int, bool, int, int, List<(int, int, int)>)? step;
		for (final candidate in _colourPixelsFlippedEventProjectionSteps) {
			if (candidate.$1 == version) {
				step = candidate;
				break;
			}
		}
		if (step == null) {
			throw RangeError(
				'event migration version mismatch: expected 0, received $version (this client has no checked-in projection for it)',
			);
		}
		if (!step.$3) {
			throw RangeError(
				'event migration version mismatch: expected 0, received ${step.$1} (the v${step.$1} to v${step.$2} transition is manual, so only an on-chain projection or a client generated from that schema can represent it)',
			);
		}
		if (payload.length != step.$4) {
			throw RangeError(
				'event migration version mismatch: expected 0, received $version (the log length does not match the v$version schema)',
			);
		}
		final destination = Uint8List(step.$5);
		for (final (sourceOffset, destinationOffset, size) in step.$6) {
			destination.setRange(destinationOffset, destinationOffset + size, payload, sourceOffset);
		}
		payload = destination;
		version = step.$2;
	}

	final projected = Uint8List(2 + payload.length);
	projected.setRange(0, 1, _colourPixelsFlippedEventEventDiscriminatorBytes);
	projected[1] = 0;
	projected.setRange(2, projected.length, payload);
	return projected;
}
/// A decoded `ColourPixelsFlippedEvent` log record.
typedef DecodedColourPixelsFlippedEventEvent = NormalizedColourPixelsFlippedEventEvent;

/// Decode a `Program data:` log line, or return null when the line is not
/// this event.
NormalizedColourPixelsFlippedEventEvent? parseColourPixelsFlippedEventEventFromLog(String log) {
	final bytes = decodeProgramDataLog(log);
	if (bytes == null || bytes.length < 1) {
		return null;
	}
	for (var index = 0; index < 1; index++) {
		if (bytes[index] != _colourPixelsFlippedEventEventDiscriminatorBytes[index]) {
			return null;
		}
	}
	return normalizeColourPixelsFlippedEventEvent(bytes);
}
