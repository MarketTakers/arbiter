import 'package:arbiter/features/connection/connection.dart';
import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:fixnum/fixnum.dart';
import 'package:protobuf/well_known_types/google/protobuf/timestamp.pb.dart';

Future<List<GrantEntry>> listEvmGrants(Connection connection) async {
  final request = EvmGrantListRequest();

  final response = await connection.request(
    UserAgentRequest(evmGrantList: request),
  );
  if (!response.hasEvmGrantList()) {
    throw Exception(
      'Expected EVM grant list response, got ${response.whichPayload()}',
    );
  }

  final result = response.evmGrantList;
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
  required int clientId,
  required int walletId,
  required Int64 chainId,
  DateTime? validFrom,
  DateTime? validUntil,
  List<int>? maxGasFeePerGas,
  List<int>? maxPriorityFeePerGas,
  TransactionRateLimit? rateLimit,
  required SpecificGrant specific,
}) async {
  throw UnimplementedError('EVM grant creation is not yet implemented.');
}

Future<void> deleteEvmGrant(Connection connection, int grantId) async {
  final response = await connection.request(
    UserAgentRequest(evmGrantDelete: EvmGrantDeleteRequest(grantId: grantId)),
  );
  if (!response.hasEvmGrantDelete()) {
    throw Exception(
      'Expected EVM grant delete response, got ${response.whichPayload()}',
    );
  }

  final result = response.evmGrantDelete;
  switch (result.whichResult()) {
    case EvmGrantDeleteResponse_Result.ok:
      return;
    case EvmGrantDeleteResponse_Result.error:
      throw Exception(_describeGrantError(result.error));
    case EvmGrantDeleteResponse_Result.notSet:
      throw Exception('Grant revoke returned no result.');
  }
}

Timestamp _toTimestamp(DateTime value) {
  final utc = value.toUtc();
  return Timestamp()
    ..seconds = Int64(utc.millisecondsSinceEpoch ~/ 1000)
    ..nanos = (utc.microsecondsSinceEpoch % 1000000) * 1000;
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
