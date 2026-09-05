import 'dart:convert';
import 'dart:typed_data';

import 'package:bs58/bs58.dart';
import 'package:ed25519_edwards/ed25519_edwards.dart' as ed25519;

const solanaPublicKeyLength = 32;
const solanaSignatureLength = 64;

Uint8List decodeBase58PublicKey(String value) {
  final Uint8List bytes;
  try {
    bytes = base58.decoder.convert(value.trim());
  } on Object {
    throw const FormatException('Invalid Solana wallet address.');
  }
  if (bytes.length != solanaPublicKeyLength) {
    throw const FormatException('Invalid Solana wallet address.');
  }
  return bytes;
}

Uint8List decodeBase64Signature(String value) {
  final normalized = value.trim();
  if (normalized.length > 128) {
    throw const FormatException('Invalid Solana signature.');
  }
  final bytes = base64Decode(normalized);
  if (bytes.length != solanaSignatureLength) {
    throw const FormatException('Invalid Solana signature.');
  }
  return bytes;
}

bool verifySolanaSignature({
  required Uint8List publicKeyBytes,
  required String message,
  required Uint8List signatureBytes,
}) {
  if (publicKeyBytes.length != solanaPublicKeyLength ||
      signatureBytes.length != solanaSignatureLength) {
    return false;
  }
  try {
    return ed25519.verify(
      ed25519.PublicKey(publicKeyBytes),
      Uint8List.fromList(utf8.encode(message)),
      signatureBytes,
    );
  } on Object {
    return false;
  }
}
