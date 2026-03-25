import 'package:arbiter/proto/client.pb.dart';
import 'package:arbiter/proto/evm.pb.dart';
import 'package:arbiter/proto/user_agent.pb.dart';
import 'package:arbiter/providers/evm/evm.dart';
import 'package:arbiter/providers/sdk_clients/list.dart';
import 'package:arbiter/providers/sdk_clients/wallet_access.dart';
import 'package:arbiter/screens/dashboard/clients/details/client_details.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';

class _FakeEvm extends Evm {
  _FakeEvm(this.wallets);

  final List<WalletEntry> wallets;

  @override
  Future<List<WalletEntry>?> build() async => wallets;
}

class _FakeWalletAccessRepository implements ClientWalletAccessRepository {
  @override
  Future<Set<int>> fetchSelectedWalletIds(int clientId) async => {1};

  @override
  Future<void> saveSelectedWalletIds(int clientId, Set<int> walletIds) async {}
}

void main() {
  testWidgets('renders client summary and wallet access controls', (
    tester,
  ) async {
    final client = SdkClientEntry(
      id: 42,
      createdAt: 1,
      info: ClientInfo(
        name: 'Safe Wallet SDK',
        version: '1.3.0',
        description: 'Primary signing client',
      ),
      pubkey: List.filled(32, 17),
    );

    final wallets = [
      WalletEntry(address: List.filled(20, 1)),
      WalletEntry(address: List.filled(20, 2)),
    ];

    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          sdkClientsProvider.overrideWith((ref) async => [client]),
          evmProvider.overrideWith(() => _FakeEvm(wallets)),
          clientWalletAccessRepositoryProvider.overrideWithValue(
            _FakeWalletAccessRepository(),
          ),
        ],
        child: const MaterialApp(home: ClientDetailsScreen(clientId: 42)),
      ),
    );

    await tester.pumpAndSettle();

    expect(find.text('Safe Wallet SDK'), findsOneWidget);
    expect(find.text('Wallet access'), findsOneWidget);
    expect(find.textContaining('0x0101'), findsOneWidget);
    expect(find.widgetWithText(FilledButton, 'Save changes'), findsOneWidget);
  });
}
