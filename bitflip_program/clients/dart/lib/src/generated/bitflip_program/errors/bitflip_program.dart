// Auto-generated. Do not edit.
// ignore_for_file: type=lint, constant_identifier_names

/// Error codes for the BitflipProgram program.

const int bitflipProgramErrorUnauthorized = 0x0; // 0

const int bitflipProgramErrorInvalidConfiguration = 0x1; // 1

const int bitflipProgramErrorInvalidGameIndex = 0x2; // 2

const int bitflipProgramErrorGameNotLive = 0x3; // 3

const int bitflipProgramErrorGameNotStarted = 0x4; // 4

const int bitflipProgramErrorInvalidSectionIndex = 0x5; // 5

const int bitflipProgramErrorSectionLocked = 0x6; // 6

const int bitflipProgramErrorInvalidFlipCount = 0x7; // 7

const int bitflipProgramErrorInvalidCoordinate = 0x8; // 8

const int bitflipProgramErrorDuplicateCoordinate = 0x9; // 9

const int bitflipProgramErrorPriceSlippage = 0xa; // 10

const int bitflipProgramErrorSectionNotActive = 0xb; // 11

const int bitflipProgramErrorSectionNotSealed = 0xc; // 12

const int bitflipProgramErrorSectionAlreadyMinted = 0xd; // 13

const int bitflipProgramErrorInvalidAsset = 0xe; // 14

const int bitflipProgramErrorInsufficientFunds = 0xf; // 15

const int bitflipProgramErrorInvalidSalePrice = 0x10; // 16

const int bitflipProgramErrorSectionNotForSale = 0x11; // 17

const int bitflipProgramErrorSectionNotTransferable = 0x12; // 18

const int bitflipProgramErrorCannotPurchaseOwnSection = 0x13; // 19

const int bitflipProgramErrorOwnerChanged = 0x14; // 20

const int bitflipProgramErrorInvalidControllerTimestamp = 0x15; // 21

const int bitflipProgramErrorInvalidControllerState = 0x16; // 22

const int bitflipProgramErrorCustodyAlreadyConfigured = 0x17; // 23

const int bitflipProgramErrorCustodyNotConfigured = 0x18; // 24

const int bitflipProgramErrorInvalidBitMint = 0x19; // 25

const int bitflipProgramErrorInvalidBitTokenAccount = 0x1a; // 26

const int bitflipProgramErrorSectionVaultAlreadyFunded = 0x1b; // 27

const int bitflipProgramErrorStalePriceWindow = 0x1c; // 28

const int bitflipProgramErrorInsufficientReward = 0x1d; // 29

const int bitflipProgramErrorNoOwnerFees = 0x1e; // 30

const int bitflipProgramErrorInvalidSectionPolicy = 0x1f; // 31

const int bitflipProgramErrorSectionPolicyLocked = 0x20; // 32

const int bitflipProgramErrorSectionPolicyChanged = 0x21; // 33

/// Map of error codes to human-readable messages.
const Map<int, String> _bitflipProgramErrorMessages = {
    bitflipProgramErrorUnauthorized: '',
    bitflipProgramErrorInvalidConfiguration: '',
    bitflipProgramErrorInvalidGameIndex: '',
    bitflipProgramErrorGameNotLive: '',
    bitflipProgramErrorGameNotStarted: '',
    bitflipProgramErrorInvalidSectionIndex: '',
    bitflipProgramErrorSectionLocked: '',
    bitflipProgramErrorInvalidFlipCount: '',
    bitflipProgramErrorInvalidCoordinate: '',
    bitflipProgramErrorDuplicateCoordinate: '',
    bitflipProgramErrorPriceSlippage: '',
    bitflipProgramErrorSectionNotActive: '',
    bitflipProgramErrorSectionNotSealed: '',
    bitflipProgramErrorSectionAlreadyMinted: '',
    bitflipProgramErrorInvalidAsset: '',
    bitflipProgramErrorInsufficientFunds: '',
    bitflipProgramErrorInvalidSalePrice: '',
    bitflipProgramErrorSectionNotForSale: '',
    bitflipProgramErrorSectionNotTransferable: '',
    bitflipProgramErrorCannotPurchaseOwnSection: '',
    bitflipProgramErrorOwnerChanged: '',
    bitflipProgramErrorInvalidControllerTimestamp: '',
    bitflipProgramErrorInvalidControllerState: '',
    bitflipProgramErrorCustodyAlreadyConfigured: '',
    bitflipProgramErrorCustodyNotConfigured: '',
    bitflipProgramErrorInvalidBitMint: '',
    bitflipProgramErrorInvalidBitTokenAccount: '',
    bitflipProgramErrorSectionVaultAlreadyFunded: '',
    bitflipProgramErrorStalePriceWindow: '',
    bitflipProgramErrorInsufficientReward: '',
    bitflipProgramErrorNoOwnerFees: '',
    bitflipProgramErrorInvalidSectionPolicy: '',
    bitflipProgramErrorSectionPolicyLocked: '',
    bitflipProgramErrorSectionPolicyChanged: '',
};

/// Get the error message for a BitflipProgram program error code.
String? getBitflipProgramErrorMessage(int code) {
  return _bitflipProgramErrorMessages[code];
}

/// Check if an error code belongs to the BitflipProgram program.
bool isBitflipProgramError(int code) {
  return _bitflipProgramErrorMessages.containsKey(code);
}
