import 'package:arbiter/features/connection/connection.dart';
import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/proto/user_agent/evm.pb.dart' as ua_evm;
import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:protobuf/well_known_types/google/protobuf/empty.pb.dart';

Future<List<WalletEntry>> listEvmWallets(Connection connection) async {
  final response = await connection.ask(
    UserAgentRequest(evm: ua_evm.Request(walletList: Empty())),
  );
  if (!response.hasEvm()) {
    throw Exception('Expected EVM response, got ${response.whichPayload()}');
  }

  final evmResponse = response.evm;
  if (!evmResponse.hasWalletList()) {
    throw Exception(
      'Expected EVM wallet list response, got ${evmResponse.whichPayload()}',
    );
  }

  final result = evmResponse.walletList;
  switch (result.whichResult()) {
    case WalletListResponse_Result.wallets:
      return result.wallets.wallets.toList(growable: false);
    case WalletListResponse_Result.error:
      throw Exception(_describeEvmError(result.error));
    case WalletListResponse_Result.notSet:
      throw Exception('EVM wallet list response was empty.');
  }
}

Future<void> createEvmWallet(Connection connection) async {
  final response = await connection.ask(
    UserAgentRequest(evm: ua_evm.Request(walletCreate: Empty())),
  );
  if (!response.hasEvm()) {
    throw Exception('Expected EVM response, got ${response.whichPayload()}');
  }

  final evmResponse = response.evm;
  if (!evmResponse.hasWalletCreate()) {
    throw Exception(
      'Expected EVM wallet create response, got ${evmResponse.whichPayload()}',
    );
  }

  final result = evmResponse.walletCreate;
  switch (result.whichResult()) {
    case WalletCreateResponse_Result.wallet:
      return;
    case WalletCreateResponse_Result.error:
      throw Exception(_describeEvmError(result.error));
    case WalletCreateResponse_Result.notSet:
      throw Exception('Wallet creation returned no result.');
  }
}

String _describeEvmError(EvmError error) {
  return switch (error) {
    EvmError.EVM_ERROR_VAULT_SEALED =>
      'The vault is sealed. Unseal it before using EVM wallets.',
    EvmError.EVM_ERROR_INTERNAL || EvmError.EVM_ERROR_UNSPECIFIED =>
      'The server failed to process the EVM request.',
    _ => 'The server failed to process the EVM request.',
  };
}
