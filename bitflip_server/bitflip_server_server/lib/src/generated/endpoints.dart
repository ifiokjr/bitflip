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
import 'package:serverpod/serverpod.dart' as _is;
import '../minting/mint_endpoint.dart' as _iy6fwn2n;

class Endpoints extends _is.EndpointDispatch {
  @override
  void initializeEndpoints(_is.Server server) {
    var endpoints = <String, _is.Endpoint>{
      'mint': _iy6fwn2n.MintEndpoint()
        ..initialize(
          server,
          'mint',
          null,
        ),
    };
    connectors['mint'] = _is.EndpointConnector(
      name: 'mint',
      endpoint: endpoints['mint']!,
      methodConnectors: {
        'createChallenge': _is.MethodConnector(
          name: 'createChallenge',
          params: {
            'walletAddress': _is.ParameterDescription(
              name: 'walletAddress',
              type: _is.getType<String>(),
              nullable: false,
            ),
            'gameIndex': _is.ParameterDescription(
              name: 'gameIndex',
              type: _is.getType<int>(),
              nullable: false,
            ),
            'sectionIndex': _is.ParameterDescription(
              name: 'sectionIndex',
              type: _is.getType<int>(),
              nullable: false,
            ),
          },
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async =>
                  (endpoints['mint'] as _iy6fwn2n.MintEndpoint).createChallenge(
                    session,
                    walletAddress: params['walletAddress'],
                    gameIndex: params['gameIndex'],
                    sectionIndex: params['sectionIndex'],
                  ),
        ),
        'mintSection': _is.MethodConnector(
          name: 'mintSection',
          params: {
            'walletAddress': _is.ParameterDescription(
              name: 'walletAddress',
              type: _is.getType<String>(),
              nullable: false,
            ),
            'gameIndex': _is.ParameterDescription(
              name: 'gameIndex',
              type: _is.getType<int>(),
              nullable: false,
            ),
            'sectionIndex': _is.ParameterDescription(
              name: 'sectionIndex',
              type: _is.getType<int>(),
              nullable: false,
            ),
            'nonce': _is.ParameterDescription(
              name: 'nonce',
              type: _is.getType<String>(),
              nullable: false,
            ),
            'signatureBase64': _is.ParameterDescription(
              name: 'signatureBase64',
              type: _is.getType<String>(),
              nullable: false,
            ),
          },
          call:
              (
                _is.Session session,
                Map<String, dynamic> params,
              ) async =>
                  (endpoints['mint'] as _iy6fwn2n.MintEndpoint).mintSection(
                    session,
                    walletAddress: params['walletAddress'],
                    gameIndex: params['gameIndex'],
                    sectionIndex: params['sectionIndex'],
                    nonce: params['nonce'],
                    signatureBase64: params['signatureBase64'],
                  ),
        ),
      },
    );
  }
}
