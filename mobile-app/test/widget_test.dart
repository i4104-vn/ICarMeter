import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:icarmeter_companion/main.dart';
import 'package:icarmeter_companion/services/storage/storage_service.dart';
import 'package:icarmeter_companion/services/app_state.dart';

void main() {
  testWidgets('ICarMeterApp smoke test', (WidgetTester tester) async {
    SharedPreferences.setMockInitialValues({});
    final storage = await StorageService.init();
    final appState = AppState(storage);

    await tester.pumpWidget(ICarMeterApp(appState: appState));
    expect(find.byType(ICarMeterApp), findsOneWidget);
  });
}
