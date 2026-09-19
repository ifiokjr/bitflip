// Auto-generated. Do not edit.
// ignore_for_file: type=lint, constant_identifier_names

/// Error codes for the BitflipProgram program.

/// The signer is not authorized for the requested state transition.
/// Message: "The signer is not authorized for the requested state transition."
const int bitflipProgramErrorUnauthorized = 0x0; // 0

/// Stored or proposed protocol configuration violates an invariant.
/// Message: "Stored or proposed protocol configuration violates an invariant."
const int bitflipProgramErrorInvalidConfiguration = 0x1; // 1

/// The game index is outside the fixed game range or creation order.
/// Message: "The game index is outside the fixed game range or creation order."
const int bitflipProgramErrorInvalidGameIndex = 0x2; // 2

/// The game cannot accept the requested operation in its current state.
/// Message: "The game cannot accept the requested operation in its current state."
const int bitflipProgramErrorGameNotLive = 0x3; // 3

/// The game launch time has not been reached.
/// Message: "The game launch time has not been reached."
const int bitflipProgramErrorGameNotStarted = 0x4; // 4

/// The section index or section account does not match the instruction.
/// Message: "The section index or section account does not match the instruction."
const int bitflipProgramErrorInvalidSectionIndex = 0x5; // 5

/// The next section has not met its time or activity unlock condition.
/// Message: "The next section has not met its time or activity unlock condition."
const int bitflipProgramErrorSectionLocked = 0x6; // 6

/// The flip batch is empty or exceeds the per-transaction bound.
/// Message: "The flip batch is empty or exceeds the per-transaction bound."
const int bitflipProgramErrorInvalidFlipCount = 0x7; // 7

/// A pixel coordinate is outside the section canvas.
/// Message: "A pixel coordinate is outside the section canvas."
const int bitflipProgramErrorInvalidCoordinate = 0x8; // 8

/// A paid flip batch contains the same coordinate more than once.
/// Message: "A paid flip batch contains the same coordinate more than once."
const int bitflipProgramErrorDuplicateCoordinate = 0x9; // 9

/// The current price exceeds a limit signed by the player.
/// Message: "The current price exceeds a limit signed by the player."
const int bitflipProgramErrorPriceSlippage = 0xa; // 10

/// The section is not active.
/// Message: "The section is not active."
const int bitflipProgramErrorSectionNotActive = 0xb; // 11

/// The section has not been sealed.
/// Message: "The section has not been sealed."
const int bitflipProgramErrorSectionNotSealed = 0xc; // 12

/// A compressed-NFT receipt was already recorded for the section.
/// Message: "A compressed-NFT receipt was already recorded for the section."
const int bitflipProgramErrorSectionAlreadyMinted = 0xd; // 13

/// The proposed compressed-NFT identity is invalid.
/// Message: "The proposed compressed-NFT identity is invalid."
const int bitflipProgramErrorInvalidAsset = 0xe; // 14

/// The paying or custody account cannot cover the requested amount.
/// Message: "The paying or custody account cannot cover the requested amount."
const int bitflipProgramErrorInsufficientFunds = 0xf; // 15

/// A section listing must have a nonzero price.
/// Message: "A section listing must have a nonzero price."
const int bitflipProgramErrorInvalidSalePrice = 0x10; // 16

/// The section has no active sale listing.
/// Message: "The section has no active sale listing."
const int bitflipProgramErrorSectionNotForSale = 0x11; // 17

/// The section state does not permit ownership transfer.
/// Message: "The section state does not permit ownership transfer."
const int bitflipProgramErrorSectionNotTransferable = 0x12; // 18

/// The current owner cannot buy their own section.
/// Message: "The current owner cannot buy their own section."
const int bitflipProgramErrorCannotPurchaseOwnSection = 0x13; // 19

/// The section owner changed before a trusted receipt was recorded.
/// Message: "The section owner changed before a trusted receipt was recorded."
const int bitflipProgramErrorOwnerChanged = 0x14; // 20

/// The runtime timestamp is invalid for the price controller.
/// Message: "The runtime timestamp is invalid for the price controller."
const int bitflipProgramErrorInvalidControllerTimestamp = 0x15; // 21

/// The persisted price-controller state is invalid.
/// Message: "The persisted price-controller state is invalid."
const int bitflipProgramErrorInvalidControllerState = 0x16; // 22

/// BIT custody was already configured and is immutable.
/// Message: "BIT custody was already configured and is immutable."
const int bitflipProgramErrorCustodyAlreadyConfigured = 0x17; // 23

/// BIT custody or the section vault has not been configured.
/// Message: "BIT custody or the section vault has not been configured."
const int bitflipProgramErrorCustodyNotConfigured = 0x18; // 24

/// The BIT mint violates the zero-decimal fixed-cap contract.
/// Message: "The BIT mint violates the zero-decimal fixed-cap contract."
const int bitflipProgramErrorInvalidBitMint = 0x19; // 25

/// A BIT reserve, vault, or recipient token account is invalid.
/// Message: "A BIT reserve, vault, or recipient token account is invalid."
const int bitflipProgramErrorInvalidBitTokenAccount = 0x1a; // 26

/// The section already received its one-time BIT allocation.
/// Message: "The section already received its one-time BIT allocation."
const int bitflipProgramErrorSectionVaultAlreadyFunded = 0x1b; // 27

/// The signed quote belongs to a different controller window.
/// Message: "The signed quote belongs to a different controller window."
const int bitflipProgramErrorStalePriceWindow = 0x1c; // 28

/// The section cannot provide the minimum reward signed by the player.
/// Message: "The section cannot provide the minimum reward signed by the player."
const int bitflipProgramErrorInsufficientReward = 0x1d; // 29

/// The section owner has no accrued fees to withdraw.
/// Message: "The section owner has no accrued fees to withdraw."
const int bitflipProgramErrorNoOwnerFees = 0x1e; // 30

/// The proposed section policy is invalid or promises unsupported rewards.
/// Message: "The proposed section policy is invalid or promises unsupported rewards."
const int bitflipProgramErrorInvalidSectionPolicy = 0x1f; // 31

/// A live section policy cannot be replaced or bypassed.
/// Message: "A live section policy cannot be replaced or bypassed."
const int bitflipProgramErrorSectionPolicyLocked = 0x20; // 32

/// The signed section-policy version is stale.
/// Message: "The signed section-policy version is stale."
const int bitflipProgramErrorSectionPolicyChanged = 0x21; // 33

/// The colour is not valid for the active section mode.
/// Message: "The colour is not valid for the active section mode."
const int bitflipProgramErrorInvalidFlipColour = 0x22; // 34

/// The section has no accrued protocol fees to withdraw.
/// Message: "The section has no accrued protocol fees to withdraw."
const int bitflipProgramErrorNoProtocolFees = 0x23; // 35

/// Map of error codes to human-readable messages.
const Map<int, String> _bitflipProgramErrorMessages = {
    bitflipProgramErrorUnauthorized: 'The signer is not authorized for the requested state transition.',
    bitflipProgramErrorInvalidConfiguration: 'Stored or proposed protocol configuration violates an invariant.',
    bitflipProgramErrorInvalidGameIndex: 'The game index is outside the fixed game range or creation order.',
    bitflipProgramErrorGameNotLive: 'The game cannot accept the requested operation in its current state.',
    bitflipProgramErrorGameNotStarted: 'The game launch time has not been reached.',
    bitflipProgramErrorInvalidSectionIndex: 'The section index or section account does not match the instruction.',
    bitflipProgramErrorSectionLocked: 'The next section has not met its time or activity unlock condition.',
    bitflipProgramErrorInvalidFlipCount: 'The flip batch is empty or exceeds the per-transaction bound.',
    bitflipProgramErrorInvalidCoordinate: 'A pixel coordinate is outside the section canvas.',
    bitflipProgramErrorDuplicateCoordinate: 'A paid flip batch contains the same coordinate more than once.',
    bitflipProgramErrorPriceSlippage: 'The current price exceeds a limit signed by the player.',
    bitflipProgramErrorSectionNotActive: 'The section is not active.',
    bitflipProgramErrorSectionNotSealed: 'The section has not been sealed.',
    bitflipProgramErrorSectionAlreadyMinted: 'A compressed-NFT receipt was already recorded for the section.',
    bitflipProgramErrorInvalidAsset: 'The proposed compressed-NFT identity is invalid.',
    bitflipProgramErrorInsufficientFunds: 'The paying or custody account cannot cover the requested amount.',
    bitflipProgramErrorInvalidSalePrice: 'A section listing must have a nonzero price.',
    bitflipProgramErrorSectionNotForSale: 'The section has no active sale listing.',
    bitflipProgramErrorSectionNotTransferable: 'The section state does not permit ownership transfer.',
    bitflipProgramErrorCannotPurchaseOwnSection: 'The current owner cannot buy their own section.',
    bitflipProgramErrorOwnerChanged: 'The section owner changed before a trusted receipt was recorded.',
    bitflipProgramErrorInvalidControllerTimestamp: 'The runtime timestamp is invalid for the price controller.',
    bitflipProgramErrorInvalidControllerState: 'The persisted price-controller state is invalid.',
    bitflipProgramErrorCustodyAlreadyConfigured: 'BIT custody was already configured and is immutable.',
    bitflipProgramErrorCustodyNotConfigured: 'BIT custody or the section vault has not been configured.',
    bitflipProgramErrorInvalidBitMint: 'The BIT mint violates the zero-decimal fixed-cap contract.',
    bitflipProgramErrorInvalidBitTokenAccount: 'A BIT reserve, vault, or recipient token account is invalid.',
    bitflipProgramErrorSectionVaultAlreadyFunded: 'The section already received its one-time BIT allocation.',
    bitflipProgramErrorStalePriceWindow: 'The signed quote belongs to a different controller window.',
    bitflipProgramErrorInsufficientReward: 'The section cannot provide the minimum reward signed by the player.',
    bitflipProgramErrorNoOwnerFees: 'The section owner has no accrued fees to withdraw.',
    bitflipProgramErrorInvalidSectionPolicy: 'The proposed section policy is invalid or promises unsupported rewards.',
    bitflipProgramErrorSectionPolicyLocked: 'A live section policy cannot be replaced or bypassed.',
    bitflipProgramErrorSectionPolicyChanged: 'The signed section-policy version is stale.',
    bitflipProgramErrorInvalidFlipColour: 'The colour is not valid for the active section mode.',
    bitflipProgramErrorNoProtocolFees: 'The section has no accrued protocol fees to withdraw.',
};

/// Get the error message for a BitflipProgram program error code.
String? getBitflipProgramErrorMessage(int code) {
  return _bitflipProgramErrorMessages[code];
}

/// Check if an error code belongs to the BitflipProgram program.
bool isBitflipProgramError(int code) {
  return _bitflipProgramErrorMessages.containsKey(code);
}
