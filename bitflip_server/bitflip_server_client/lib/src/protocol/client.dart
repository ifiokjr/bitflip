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
import 'dart:async' as _ida;
import 'package:bitflip_server_client/src/protocol/minting/mint_challenge_view.dart'
    as _il1kyfw6;
import 'package:bitflip_server_client/src/protocol/minting/mint_section_result.dart'
    as _ibkzpld6;
import 'package:http/http.dart' as _i85jenna;
import 'package:serverpod_client/serverpod_client.dart' as _isc;
import 'protocol.dart' as _il2as5qe;

/// {@category Endpoint}
class EndpointMint extends _isc.EndpointRef {
  EndpointMint(_isc.EndpointCaller caller) : super(caller);

  @override
  String get name => 'mint';

  _ida.Future<_il1kyfw6.MintChallengeView> createChallenge({
    required String walletAddress,
    required int gameIndex,
    required int sectionIndex,
  }) => caller.callServerEndpoint<_il1kyfw6.MintChallengeView>(
    'mint',
    'createChallenge',
    {
      'walletAddress': walletAddress,
      'gameIndex': gameIndex,
      'sectionIndex': sectionIndex,
    },
  );

  _ida.Future<_ibkzpld6.MintSectionResult> mintSection({
    required String walletAddress,
    required int gameIndex,
    required int sectionIndex,
    required String nonce,
    required String signatureBase64,
  }) => caller.callServerEndpoint<_ibkzpld6.MintSectionResult>(
    'mint',
    'mintSection',
    {
      'walletAddress': walletAddress,
      'gameIndex': gameIndex,
      'sectionIndex': sectionIndex,
      'nonce': nonce,
      'signatureBase64': signatureBase64,
    },
  );
}

class Client extends _isc.ServerpodClientShared {
  Client(
    String host, {
    dynamic securityContext,
    Duration? streamingConnectionTimeout,
    Duration? connectionTimeout,
    Function(
      _isc.MethodCallContext,
      Object,
      StackTrace,
    )?
    onFailedCall,
    Function(_isc.MethodCallContext)? onSucceededCall,
    bool? disconnectStreamsOnLostInternetConnection,
    _i85jenna.Client? httpClientOverride,
  }) : super(
         host,
         _il2as5qe.Protocol(),
         securityContext: securityContext,
         streamingConnectionTimeout: streamingConnectionTimeout,
         connectionTimeout: connectionTimeout,
         onFailedCall: onFailedCall,
         onSucceededCall: onSucceededCall,
         disconnectStreamsOnLostInternetConnection:
             disconnectStreamsOnLostInternetConnection,
         httpClientOverride: httpClientOverride,
       ) {
    mint = EndpointMint(this);
  }

  late final EndpointMint mint;

  @override
  Map<String, _isc.EndpointRef> get endpointRefLookup => {'mint': mint};

  @override
  Map<String, _isc.ModuleEndpointCaller> get moduleLookup => {};
}
