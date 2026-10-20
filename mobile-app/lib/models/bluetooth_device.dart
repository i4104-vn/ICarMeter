class BtDeviceInfo {
  final String mac;
  final String name;
  final int rssi;
  final bool isBonded;
  final bool isConnecting;
  final bool isConnected;

  const BtDeviceInfo({
    required this.mac,
    required this.name,
    this.rssi = -100,
    this.isBonded = false,
    this.isConnecting = false,
    this.isConnected = false,
  });

  factory BtDeviceInfo.fromJson(Map<String, dynamic> json) {
    return BtDeviceInfo(
      mac: json['mac'] as String? ?? '',
      name: json['name'] as String? ?? json['mac'] as String? ?? 'Thiết bị Bluetooth',
      rssi: (json['rssi'] as num?)?.toInt() ?? -100,
      isBonded: (json['isBonded'] as bool?) ?? false,
      isConnecting: (json['isConnecting'] as bool?) ?? false,
      isConnected: (json['isConnected'] as bool?) ?? false,
    );
  }

  Map<String, dynamic> toJson() => {
        'mac': mac,
        'name': name,
        'rssi': rssi,
        'isBonded': isBonded,
        'isConnecting': isConnecting,
        'isConnected': isConnected,
      };

  BtDeviceInfo copyWith({
    String? mac,
    String? name,
    int? rssi,
    bool? isBonded,
    bool? isConnecting,
    bool? isConnected,
  }) {
    return BtDeviceInfo(
      mac: mac ?? this.mac,
      name: name ?? this.name,
      rssi: rssi ?? this.rssi,
      isBonded: isBonded ?? this.isBonded,
      isConnecting: isConnecting ?? this.isConnecting,
      isConnected: isConnected ?? this.isConnected,
    );
  }
}
