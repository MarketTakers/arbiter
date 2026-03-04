import 'package:flutter/material.dart';

class CalcScreen extends StatefulWidget {
	const CalcScreen({super.key});

	@override
	State<CalcScreen> createState() => _CalcScreenState();
}

class _CalcScreenState extends State<CalcScreen> {
	int _count = 0;

	void _increment() {
		setState(() {
			_count++;
		});
	}

	void _decrement() {
		setState(() {
			_count--;
		});
	}

	void _reset() {
		setState(() {
			_count = 0;
		});
	}

	@override
	Widget build(BuildContext context) {
		return Scaffold(
			appBar: AppBar(
				title: const Text('Counter'),
			),
			body: Center(
				child: Column(
					mainAxisSize: MainAxisSize.min,
					children: [
						const Text('Current count'),
						const SizedBox(height: 8),
						Text(
							'$_count',
							style: Theme.of(context).textTheme.displaySmall,
						),
						const SizedBox(height: 20),
						Row(
							mainAxisAlignment: MainAxisAlignment.center,
							children: [
								IconButton(
									onPressed: _decrement,
									icon: const Icon(Icons.remove_circle_outline),
									tooltip: 'Decrement',
								),
								const SizedBox(width: 12),
								ElevatedButton(
									onPressed: _reset,
									child: const Text('Reset'),
								),
								const SizedBox(width: 12),
								IconButton(
									onPressed: _increment,
									icon: const Icon(Icons.add_circle_outline),
									tooltip: 'Increment',
								),
							],
						),
					],
				),
			),
		);
	}
}
