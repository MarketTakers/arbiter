import 'package:mtcore/markettakers.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:arbiter/features/pk_manager.dart';
import 'package:arbiter/features/simple_ed25519.dart';

part 'key.g.dart';

@riverpod
KeyManager keyManager(Ref ref) {
  return SimpleEd25519Manager();
}

@Riverpod(keepAlive: true)
class Key extends _$Key {
  @override
  Future<KeyHandle?> build() async {
    final manager = ref.watch(keyManagerProvider);
    final keyHandle = await manager.get();
    return keyHandle;
  }

  Future<void> create() async {
    final manager = ref.watch(keyManagerProvider);
    if (state.value != null) {
      return;
    }

    state = await AsyncValue.guard(() async {
      final newKeyHandle = await manager.create();
      return newKeyHandle;
    });
  }
}

class KeyBootstrapper implements StageFactory {
  final ProviderContainer ref;

  KeyBootstrapper({required this.ref});

  @override
  String get title => "Setting up your identity";

  @override
  Future<bool> get isAlreadyCompleted async {
    final key = await ref.read(keyProvider.future);
    return key != null;
  }

  @override
  Future<void> start(StageController controller) async {
    controller.setIndefiniteProgress();
    final key = ref.read(keyProvider.notifier);

    await key.create();
  }

  @override
  void dispose() {}
}
