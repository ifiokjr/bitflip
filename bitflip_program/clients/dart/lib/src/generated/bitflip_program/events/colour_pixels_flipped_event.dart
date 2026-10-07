// Auto-generated. Do not edit.
// ignore_for_file: type=lint

import 'dart:typed_data';
import 'package:solana_kit_addresses/solana_kit_addresses.dart';
import 'package:solana_kit_codecs_core/solana_kit_codecs_core.dart';
import 'package:solana_kit_codecs_data_structures/solana_kit_codecs_data_structures.dart';
import 'package:solana_kit_codecs_numbers/solana_kit_codecs_numbers.dart';

import 'event_log.dart';

/// Event record `ColourPixelsFlippedEvent`.
class ColourPixelsFlippedEventEvent extends BitflipProgramEvent {
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

	@override
	String get name => 'colourPixelsFlippedEvent';

	String toString() => 'ColourPixelsFlippedEventEvent(discriminator: ${discriminator}, migrationVersion: ${migrationVersion}, player: ${player}, policyVersion: ${policyVersion}, revision: ${revision}, coordinates: ${coordinates}, gameIndex: ${gameIndex}, sectionIndex: ${sectionIndex}, count: ${count}, colour: ${colour})';
}

/// The discriminator this event is emitted under.
const colourPixelsFlippedEventEventDiscriminator = 1;

/// The discriminator bytes as stored at offset zero.
const List<int> _colourPixelsFlippedEventEventDiscriminatorBytes = [1];

/// The migration version this event decodes.
const colourPixelsFlippedEventEventMigrationVersion = 0;

/// Exact current byte length of a `ColourPixelsFlippedEvent` record, envelope included.
const colourPixelsFlippedEventEventSize = 86;

/// Decode one `ColourPixelsFlippedEvent` record.
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
				? 'event migration version mismatch: expected 0, received $v1 (decode it with the event for that version)'
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

/// A decoded `ColourPixelsFlippedEvent` log record.
typedef DecodedColourPixelsFlippedEventEvent = ColourPixelsFlippedEventEvent;

/// Decode a `Program data:` log line, or return null when the line is not
/// this event.
ColourPixelsFlippedEventEvent? parseColourPixelsFlippedEventEventFromLog(String log) {
	final bytes = decodeProgramDataLog(log);
	if (bytes == null || bytes.length < 2) {
		return null;
	}
	for (var index = 0; index < 1; index++) {
		if (bytes[index] != _colourPixelsFlippedEventEventDiscriminatorBytes[index]) {
			return null;
		}
	}
	if (bytes[1] != colourPixelsFlippedEventEventMigrationVersion) {
		return null;
	}
	return decodeColourPixelsFlippedEventEvent(bytes);
}
