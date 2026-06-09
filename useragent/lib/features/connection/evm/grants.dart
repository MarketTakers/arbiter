import 'package:arbiter/features/connection/connection.dart';
import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/proto/user_agent/evm.pb.dart' as ua_evm;
import 'package:arbiter/proto/user_agent.pb.dart';

Future<List<GrantEntry>> listEvmGrants(Connection connection) async {
  final request = EvmGrantListRequest();

  final response = await connection.ask(
    UserAgentRequest(evm: ua_evm.Request(grantList: request)),
  );
  if (!response.hasEvm()) {
    throw Exception('Expected EVM response, got ${response.whichPayload()}');
  }

  final evmResponse = response.evm;
  if (!evmResponse.hasGrantList()) {
    throw Exception(
      'Expected EVM grant list response, got ${evmResponse.whichPayload()}',
    );
  }

  final result = evmResponse.grantList;
  switch (result.whichResult()) {
    case EvmGrantListResponse_Result.grants:
      return result.grants.grants.toList(growable: false);
    case EvmGrantListResponse_Result.error:
      throw Exception(_describeGrantError(result.error));
    case EvmGrantListResponse_Result.notSet:
      throw Exception('EVM grant list response was empty.');
  }
}

Future<int> createEvmGrant(
  Connection connection, {
  required SharedSettings sharedSettings,
  required SpecificGrant specific,
}) async {
  final request = UserAgentRequest(
    evm: ua_evm.Request(
      grantCreate: EvmGrantCreateRequest(
        shared: sharedSettings,
        specific: specific,
      ),
    ),
  );

  final resp = await connection.ask(request);

  if (!resp.hasEvm()) {
    throw Exception('Expected EVM response, got ${resp.whichPayload()}');
  }

  final evmResponse = resp.evm;
  if (!evmResponse.hasGrantCreate()) {
    throw Exception(
      'Expected EVM grant create response, got ${evmResponse.whichPayload()}',
    );
  }

  final result = evmResponse.grantCreate;

  return result.grantId;
}

Future<void> deleteEvmGrant(Connection connection, int grantId) async {
  final response = await connection.ask(
    UserAgentRequest(
      evm: ua_evm.Request(grantDelete: EvmGrantDeleteRequest(grantId: grantId)),
    ),
  );
  if (!response.hasEvm()) {
    throw Exception('Expected EVM response, got ${response.whichPayload()}');
  }

  final evmResponse = response.evm;
  if (!evmResponse.hasGrantDelete()) {
    throw Exception(
      'Expected EVM grant delete response, got ${evmResponse.whichPayload()}',
    );
  }

  final result = evmResponse.grantDelete;
  switch (result.whichResult()) {
    case EvmGrantDeleteResponse_Result.ok:
      return;
    case EvmGrantDeleteResponse_Result.error:
      throw Exception(_describeGrantError(result.error));
    case EvmGrantDeleteResponse_Result.notSet:
      throw Exception('Grant revoke returned no result.');
  }
}

String _describeGrantError(EvmError error) {
  return switch (error) {
    EvmError.EVM_ERROR_VAULT_SEALED =>
      'The vault is sealed. Unseal it before using EVM grants.',
    EvmError.EVM_ERROR_INTERNAL || EvmError.EVM_ERROR_UNSPECIFIED =>
      'The server failed to process the EVM grant request.',
    _ => 'The server failed to process the EVM grant request.',
  };
}
