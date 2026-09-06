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
import 'package:bitflip_server_server/src/generated/future_calls.dart'
    as _ibxc289e;
import 'package:serverpod/serverpod.dart' as _is;
import '../colour/colour_canvas_endpoint.dart' as _ixrwsudz;
import '../minting/mint_endpoint.dart' as _iy6fwn2n;
export 'future_calls.dart' show ServerpodFutureCallsGetter;

class Endpoints extends _is.EndpointDispatch {
  @override
  void initializeEndpoints(_is.Server server) {
    var endpoints = <String, _is.Endpoint>{
      'colourCanvas': _ixrwsudz.ColourCanvasEndpoint()
        ..initialize(
          server,
          'colourCanvas',
          null,
        ),
      'mint': _iy6fwn2n.MintEndpoint()
        ..initialize(
          server,
          'mint',
          null,
        ),
    };
    connectors['colourCanvas'] = _is.EndpointConnector(
      name: 'colourCanvas',
      endpoint: endpoints['colourCanvas']!,
      methodConnectors: {
        'load': _is.MethodConnector(
          name: 'load',
          params: {
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
                  (endpoints['colourCanvas'] as _ixrwsudz.ColourCanvasEndpoint)
                      .load(
                        session,
                        gameIndex: params['gameIndex'],
                        sectionIndex: params['sectionIndex'],
                      ),
        ),
        'recordSignature': _is.MethodConnector(
          name: 'recordSignature',
          params: {
            'transactionSignature': _is.ParameterDescription(
              name: 'transactionSignature',
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
                  (endpoints['colourCanvas'] as _ixrwsudz.ColourCanvasEndpoint)
                      .recordSignature(
                        session,
                        transactionSignature: params['transactionSignature'],
                        gameIndex: params['gameIndex'],
                        sectionIndex: params['sectionIndex'],
                      ),
        ),
      },
    );
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

  @override
  _is.FutureCallDispatch? get futureCalls {
    return _ibxc289e.FutureCalls();
  }
}
