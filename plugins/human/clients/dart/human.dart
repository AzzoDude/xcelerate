// Client for the human plugin (generated).
import 'dart:convert';

const pluginName = 'human';

class Info {
  final String? name;
  final bool? enabled;
  final List<String>? ops;
  Info({this.name, this.enabled, this.ops});
  Map<String, dynamic> toJson() => {'name': name, 'enabled': enabled, 'ops': ops};
  factory Info.fromJson(Map<String, dynamic> json) => Info(name: json['name'] as String?, enabled: json['enabled'] as bool?, ops: json['ops'] as List<String>?);
}

class Move {
  final bool? moved;
  final double? x;
  final double? y;
  Move({this.moved, this.x, this.y});
  Map<String, dynamic> toJson() => {'moved': moved, 'x': x, 'y': y};
  factory Move.fromJson(Map<String, dynamic> json) => Move(moved: json['moved'] as bool?, x: json['x'] as double?, y: json['y'] as double?);
}

class Click {
  final bool? clicked;
  final double? x;
  final double? y;
  Click({this.clicked, this.x, this.y});
  Map<String, dynamic> toJson() => {'clicked': clicked, 'x': x, 'y': y};
  factory Click.fromJson(Map<String, dynamic> json) => Click(clicked: json['clicked'] as bool?, x: json['x'] as double?, y: json['y'] as double?);
}

class Type {
  final int? typed;
  Type({this.typed});
  Map<String, dynamic> toJson() => {'typed': typed};
  factory Type.fromJson(Map<String, dynamic> json) => Type(typed: json['typed'] as int?);
}

class Scroll {
  final double? scrolled;
  Scroll({this.scrolled});
  Map<String, dynamic> toJson() => {'scrolled': scrolled};
  factory Scroll.fromJson(Map<String, dynamic> json) => Scroll(scrolled: json['scrolled'] as double?);
}

class Delay {
  final int? sleptMs;
  Delay({this.sleptMs});
  Map<String, dynamic> toJson() => {'sleptMs': sleptMs};
  factory Delay.fromJson(Map<String, dynamic> json) => Delay(sleptMs: json['sleptMs'] as int?);
}

class Human {
  final dynamic browser;
  Human(this.browser);

  Future<Info> info() async {
    final args = <String, dynamic>{};
    final raw = await browser.plugin(pluginName).invoke('info', jsonEncode(args));
    return Info.fromJson(jsonDecode(raw) as Map<String, dynamic>);
  }

  Future<Move> move(double x, double y) async {
    final args = <String, dynamic>{'x': x, 'y': y};
    final raw = await browser.plugin(pluginName).invoke('move', jsonEncode(args));
    return Move.fromJson(jsonDecode(raw) as Map<String, dynamic>);
  }

  Future<Click> click(double x, double y) async {
    final args = <String, dynamic>{'x': x, 'y': y};
    final raw = await browser.plugin(pluginName).invoke('click', jsonEncode(args));
    return Click.fromJson(jsonDecode(raw) as Map<String, dynamic>);
  }

  Future<Type> type(String text) async {
    final args = <String, dynamic>{'text': text};
    final raw = await browser.plugin(pluginName).invoke('type', jsonEncode(args));
    return Type.fromJson(jsonDecode(raw) as Map<String, dynamic>);
  }

  Future<Scroll> scroll(double deltaY) async {
    final args = <String, dynamic>{'deltaY': deltaY};
    final raw = await browser.plugin(pluginName).invoke('scroll', jsonEncode(args));
    return Scroll.fromJson(jsonDecode(raw) as Map<String, dynamic>);
  }

  Future<Delay> delay({ int? minMs, int? maxMs }) async {
    final args = <String, dynamic>{if (minMs != null) 'minMs': minMs, if (maxMs != null) 'maxMs': maxMs};
    final raw = await browser.plugin(pluginName).invoke('delay', jsonEncode(args));
    return Delay.fromJson(jsonDecode(raw) as Map<String, dynamic>);
  }

}