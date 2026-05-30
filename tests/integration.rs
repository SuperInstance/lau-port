use lau_port::*;
use std::collections::HashMap;
use uuid::Uuid;

// ─── Port Protocol Tests ────────────────────────────────────────────────────

#[test]
fn test_protocol_labels() {
    assert_eq!(PortProtocol::Telegram.label(), "Telegram");
    assert_eq!(PortProtocol::Discord.label(), "Discord");
    assert_eq!(PortProtocol::WebSocket.label(), "WebSocket");
    assert_eq!(PortProtocol::StdinStdout.label(), "stdin/stdout");
    assert_eq!(PortProtocol::File("log.txt".into()).label(), "File");
    assert_eq!(PortProtocol::Custom("MyProto".into()).label(), "MyProto");
}

#[test]
fn test_protocol_network() {
    assert!(PortProtocol::Telegram.is_network());
    assert!(PortProtocol::Mqtt.is_network());
    assert!(!PortProtocol::Serial.is_network());
    assert!(!PortProtocol::StdinStdout.is_network());
    assert!(!PortProtocol::File("x".into()).is_network());
}

#[test]
fn test_protocol_hardware() {
    assert!(PortProtocol::Serial.is_hardware());
    assert!(PortProtocol::Gpio.is_hardware());
    assert!(PortProtocol::I2c.is_hardware());
    assert!(PortProtocol::Spi.is_hardware());
    assert!(!PortProtocol::Http.is_hardware());
    assert!(!PortProtocol::File("x".into()).is_hardware());
}

#[test]
fn test_protocol_filesystem() {
    assert!(PortProtocol::File("log.txt".into()).is_filesystem());
    assert!(!PortProtocol::Telegram.is_filesystem());
    assert!(!PortProtocol::Serial.is_filesystem());
}

// ─── Port Direction Tests ───────────────────────────────────────────────────

#[test]
fn test_direction_can_send() {
    assert!(!PortDirection::Inbound.can_send());
    assert!(PortDirection::Outbound.can_send());
    assert!(PortDirection::Bidirectional.can_send());
}

#[test]
fn test_direction_can_receive() {
    assert!(PortDirection::Inbound.can_receive());
    assert!(!PortDirection::Outbound.can_receive());
    assert!(PortDirection::Bidirectional.can_receive());
}

#[test]
fn test_direction_serialize_roundtrip() {
    let dirs = vec![
        PortDirection::Inbound,
        PortDirection::Outbound,
        PortDirection::Bidirectional,
    ];
    for d in &dirs {
        let json = serde_json::to_string(d).unwrap();
        let back: PortDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(*d, back);
    }
}

// ─── Message Content Tests ──────────────────────────────────────────────────

#[test]
fn test_message_content_text() {
    let c = MessageContent::text("hello");
    assert!(c.is_text());
    assert!(!c.is_command());
    assert_eq!(c.as_text(), Some("hello"));
}

#[test]
fn test_message_content_command() {
    let c = MessageContent::command("ping", &["--count", "3"]);
    assert!(!c.is_text());
    assert!(c.is_command());
    assert_eq!(c.as_text(), None);
    match c {
        MessageContent::Command { ref name, ref args } => {
            assert_eq!(name, "ping");
            assert_eq!(args, &["--count", "3"]);
        }
        _ => panic!("expected Command"),
    }
}

#[test]
fn test_message_content_html() {
    let c = MessageContent::Html("<b>bold</b>".to_string());
    assert!(!c.is_text());
    assert!(c.as_text().is_none());
}

#[test]
fn test_message_content_markdown() {
    let c = MessageContent::Markdown("# hello".to_string());
    assert!(!c.is_text());
    assert!(c.as_text().is_none());
}

// ─── InboundMessage Tests ──────────────────────────────────────────────────

#[test]
fn test_inbound_message_text_constructor() {
    let msg = InboundMessage::text("hello world");
    assert!(msg.is_text());
    assert!(!msg.is_command());
    assert_eq!(msg.as_text(), Some("hello world"));
    assert!(msg.sender.is_none());
    assert!(msg.reply_to.is_none());
    assert!(msg.metadata.is_empty());
}

#[test]
fn test_inbound_message_command_constructor() {
    let msg = InboundMessage::command("deploy", &["--env", "prod"]);
    assert!(msg.is_command());
    assert!(!msg.is_text());
    assert!(msg.as_text().is_none());
}

// ─── OutboundMessage Tests ─────────────────────────────────────────────────

#[test]
fn test_outbound_message_text_constructor() {
    let msg = OutboundMessage::text("hi there");
    assert!(msg.reply_to.is_none());
    assert!(msg.target.is_none());
}

#[test]
fn test_outbound_message_reply_constructor() {
    let msg = OutboundMessage::reply("msg-123", "got it");
    assert_eq!(msg.reply_to, Some("msg-123".to_string()));
    assert!(msg.target.is_none());
}

#[test]
fn test_outbound_message_content_types() {
    let msg = OutboundMessage::text("hello");
    assert!(matches!(msg.content, MessageContent::Text(_)));
}

// ─── PortConfig Tests ───────────────────────────────────────────────────────

#[test]
fn test_port_config_from_json() {
    let json = r#"{
        "id": "my-bot",
        "protocol": "Telegram",
        "direction": "Bidirectional",
        "params": {
            "bot_token_ref": "TELEGRAM_BOT_TOKEN",
            "chat_id": "-1001234567890"
        },
        "enabled": true,
        "max_rate": 30
    }"#;
    let config = PortConfig::from_json(json).unwrap();
    assert_eq!(config.id, "my-bot");
    assert_eq!(config.protocol, PortProtocol::Telegram);
    assert_eq!(config.direction, PortDirection::Bidirectional);
    assert!(config.enabled);
    assert_eq!(config.max_rate, Some(30));
    assert_eq!(
        config.params.get("bot_token_ref").unwrap(),
        "TELEGRAM_BOT_TOKEN"
    );
}

#[test]
fn test_port_config_to_json_roundtrip() {
    let mut params = HashMap::new();
    params.insert("device".to_string(), "/dev/ttyUSB0".to_string());
    params.insert("baud_rate".to_string(), "115200".to_string());
    let config = PortConfig {
        id: "serial-1".to_string(),
        protocol: PortProtocol::Serial,
        direction: PortDirection::Bidirectional,
        params,
        enabled: true,
        max_rate: None,
        deadband: None,
    };
    let json = config.to_json();
    let parsed = PortConfig::from_json(&json).unwrap();
    assert_eq!(parsed.id, "serial-1");
    assert_eq!(parsed.protocol, PortProtocol::Serial);
}

#[test]
fn test_port_config_invalid_json() {
    let result = PortConfig::from_json("not valid json");
    assert!(result.is_err());
    match result {
        Err(PortError::ConfigInvalid(_)) => {} // expected
        _ => panic!("expected ConfigInvalid error"),
    }
}

#[test]
fn test_port_config_with_deadband() {
    let json = r#"{
        "id": "temp-sensor",
        "protocol": {"Custom": "DS18B20"},
        "direction": "Inbound",
        "params": {},
        "enabled": true,
        "deadband": {
            "lower": 18.0,
            "upper": 26.0,
            "check_interval_ms": 5000
        }
    }"#;
    let config = PortConfig::from_json(json).unwrap();
    let db = config.deadband.unwrap();
    assert_eq!(db.lower, 18.0);
    assert_eq!(db.upper, 26.0);
    assert_eq!(db.check_interval_ms, 5000);
}

// ─── PortError Tests ────────────────────────────────────────────────────────

#[test]
fn test_port_error_display() {
    let err = PortError::NotConnected("port is down".into());
    assert_eq!(err.to_string(), "not connected: port is down");

    let err = PortError::RateLimited { retry_after_ms: 5000 };
    assert_eq!(err.to_string(), "rate limited, retry after 5000ms");

    let err = PortError::SendFailed("timeout".into());
    assert_eq!(err.to_string(), "send failed: timeout");

    let err = PortError::Timeout(30000);
    assert_eq!(err.to_string(), "timeout after 30000ms");

    let err = PortError::ConnectFailed("permission denied".into());
    assert_eq!(err.to_string(), "connect failed: permission denied");
}

// ─── MemoryPort Tests ──────────────────────────────────────────────────────

#[test]
fn test_memory_port_connect_disconnect() {
    let mut port = MemoryPort::new("mem1", PortProtocol::Custom("test".into()), PortDirection::Bidirectional);
    assert!(!port.is_connected());
    port.connect().unwrap();
    assert!(port.is_connected());
    port.disconnect().unwrap();
    assert!(!port.is_connected());
}

#[test]
fn test_memory_port_send_receive() {
    let mut port = MemoryPort::new("mem1", PortProtocol::Custom("test".into()), PortDirection::Bidirectional);
    port.connect().unwrap();

    let msg = OutboundMessage::text("hello from test");
    let result = port.send(msg);
    assert!(result.is_ok());

    let outbox = port.drain_outbox();
    assert_eq!(outbox.len(), 1);
    assert_eq!(outbox[0].content.as_text(), Some("hello from test"));
}

#[test]
fn test_memory_port_receive_injected() {
    let mut port = MemoryPort::new("mem1", PortProtocol::Custom("test".into()), PortDirection::Bidirectional);
    port.connect().unwrap();

    let inbound = InboundMessage::text("incoming!");
    port.inject(inbound);

    let received = port.receive().unwrap();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].as_text(), Some("incoming!"));
}

#[test]
fn test_memory_port_send_fails_when_not_connected() {
    let port = MemoryPort::new("mem1", PortProtocol::Custom("test".into()), PortDirection::Bidirectional);
    let result = port.send(OutboundMessage::text("fail"));
    assert!(result.is_err());
    match result {
        Err(PortError::NotConnected(_)) => {}
        _ => panic!("expected NotConnected"),
    }
}

#[test]
fn test_memory_port_receive_fails_when_not_connected() {
    let port = MemoryPort::new("mem1", PortProtocol::Custom("test".into()), PortDirection::Bidirectional);
    let result = port.receive();
    assert!(result.is_err());
}

#[test]
fn test_memory_port_send_fails_on_inbound_only() {
    let mut port = MemoryPort::new("mem1", PortProtocol::Custom("test".into()), PortDirection::Inbound);
    port.connect().unwrap();
    let result = port.send(OutboundMessage::text("should fail"));
    assert!(result.is_err());
    match result {
        Err(PortError::SendFailed(_)) => {}
        _ => panic!("expected SendFailed"),
    }
}

#[test]
fn test_memory_port_receive_fails_on_outbound_only() {
    let mut port = MemoryPort::new("mem1", PortProtocol::Custom("test".into()), PortDirection::Outbound);
    port.connect().unwrap();
    let result = port.receive();
    assert!(result.is_err());
}

#[test]
fn test_memory_port_stats() {
    let mut port = MemoryPort::new("mem1", PortProtocol::Custom("test".into()), PortDirection::Bidirectional);
    port.connect().unwrap();

    let stats = port.stats();
    assert_eq!(stats.messages_sent, 0);
    assert_eq!(stats.messages_received, 0);

    port.send(OutboundMessage::text("msg1")).unwrap();
    let stats = port.stats();
    assert_eq!(stats.messages_sent, 1);
    assert!(stats.last_activity.is_some());
}

#[test]
fn test_memory_port_multiple_messages() {
    let mut port = MemoryPort::new("mem1", PortProtocol::Custom("test".into()), PortDirection::Bidirectional);
    port.connect().unwrap();

    for i in 0..5 {
        let msg = OutboundMessage::text(&format!("msg {i}"));
        port.send(msg).unwrap();
    }

    let outbox = port.drain_outbox();
    assert_eq!(outbox.len(), 5);

    for i in 0..5 {
        port.inject(InboundMessage::text(&format!("incoming {i}")));
    }

    let received = port.receive().unwrap();
    assert_eq!(received.len(), 5);
    assert_eq!(received[2].as_text(), Some("incoming 2"));
}

// ─── StdioPort Tests ───────────────────────────────────────────────────────

#[test]
fn test_stdio_port_basics() {
    let mut port = StdioPort::new("stdio");
    assert_eq!(port.id(), "stdio");
    assert_eq!(port.protocol(), PortProtocol::StdinStdout);
    assert!(!port.is_connected());
    port.connect().unwrap();
    assert!(port.is_connected());
    port.disconnect().unwrap();
    assert!(!port.is_connected());
}

#[test]
fn test_stdio_port_send_fails_when_disconnected() {
    let port = StdioPort::new("stdio");
    let result = port.send(OutboundMessage::text("test"));
    assert!(result.is_err());
    match result {
        Err(PortError::NotConnected(_)) => {}
        _ => panic!("expected NotConnected"),
    }
}

#[test]
fn test_stdio_port_receive_returns_not_connected() {
    let mut port = StdioPort::new("stdio");
    port.connect().unwrap();
    let result = port.receive();
    assert!(result.is_err());
}

// ─── PortRegistry Tests ────────────────────────────────────────────────────

#[test]
fn test_registry_new_is_empty() {
    let registry = PortRegistry::new();
    assert!(registry.list().is_empty());
}

#[test]
fn test_registry_register_and_list() {
    let mut registry = PortRegistry::new();
    let port = MemoryPort::new("mem1", PortProtocol::Custom("test".into()), PortDirection::Bidirectional);
    registry.register(Box::new(port));
    let list = registry.list();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].0, "mem1");
    assert!(!list[0].3); // not connected
}

#[test]
fn test_registry_unregister() {
    let mut registry = PortRegistry::new();
    let port = MemoryPort::new("mem1", PortProtocol::Custom("test".into()), PortDirection::Bidirectional);
    registry.register(Box::new(port));
    let removed = registry.unregister("mem1").unwrap();
    assert_eq!(removed.id(), "mem1");
    assert!(registry.list().is_empty());
}

#[test]
fn test_registry_unregister_not_found() {
    let mut registry = PortRegistry::new();
    let result = registry.unregister("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_registry_send_to_nonexistent() {
    let registry = PortRegistry::new();
    let result = registry.send("ghost", OutboundMessage::text("hello"));
    assert!(result.is_err());
}

#[test]
fn test_registry_send_to_connected_port() {
    let mut registry = PortRegistry::new();
    let mut port = MemoryPort::new("mem1", PortProtocol::Custom("test".into()), PortDirection::Bidirectional);
    port.connect().unwrap();
    registry.register(Box::new(port));
    let result = registry.send("mem1", OutboundMessage::text("hello"));
    assert!(result.is_ok());
}

#[test]
fn test_registry_receive_all() {
    let mut registry = PortRegistry::new();
    let mut port = MemoryPort::new("mem1", PortProtocol::Custom("test".into()), PortDirection::Bidirectional);
    port.connect().unwrap();
    port.inject(InboundMessage::text("hello"));
    registry.register(Box::new(port));
    let results = registry.receive_all();
    assert_eq!(results.len(), 1);
    let (id, msgs) = &results[0];
    assert_eq!(id, "mem1");
    assert!(msgs.is_ok());
    assert_eq!(msgs.as_ref().unwrap().len(), 1);
}

#[test]
fn test_registry_connect_all() {
    let mut registry = PortRegistry::new();
    let port1 = MemoryPort::new("mem1", PortProtocol::Custom("test".into()), PortDirection::Bidirectional);
    let port2 = MemoryPort::new("mem2", PortProtocol::Custom("test".into()), PortDirection::Bidirectional);
    registry.register(Box::new(port1));
    registry.register(Box::new(port2));

    let results = registry.connect_all();
    assert_eq!(results.len(), 2);
    for (id, result) in &results {
        assert!(result.is_ok(), "port {id} should connect");
    }

    let list = registry.list();
    for (_, _, _, connected) in &list {
        assert!(connected);
    }
}

#[test]
fn test_registry_disconnect_all() {
    let mut registry = PortRegistry::new();
    let mut port1 = MemoryPort::new("mem1", PortProtocol::Custom("test".into()), PortDirection::Bidirectional);
    port1.connect().unwrap();
    registry.register(Box::new(port1));

    registry.disconnect_all();
    let list = registry.list();
    for (_, _, _, connected) in &list {
        assert!(!connected);
    }
}

#[test]
fn test_registry_health_check() {
    let mut registry = PortRegistry::new();
    let mut port1 = MemoryPort::new("mem1", PortProtocol::Custom("test".into()), PortDirection::Bidirectional);
    port1.connect().unwrap();
    let port2 = MemoryPort::new("mem2", PortProtocol::Custom("test".into()), PortDirection::Bidirectional);
    registry.register(Box::new(port1));
    registry.register(Box::new(port2));

    let health = registry.health_check();
    assert_eq!(health.get("mem1"), Some(&true));
    assert_eq!(health.get("mem2"), Some(&false));
}

#[test]
fn test_registry_multiple_ports() {
    let mut registry = PortRegistry::new();
    let p1 = MemoryPort::new("telegram", PortProtocol::Telegram, PortDirection::Bidirectional);
    let p2 = MemoryPort::new("mqtt", PortProtocol::Mqtt, PortDirection::Inbound);
    let p3 = MemoryPort::new("serial", PortProtocol::Serial, PortDirection::Bidirectional);
    registry.register(Box::new(p1));
    registry.register(Box::new(p2));
    registry.register(Box::new(p3));

    let list = registry.list();
    assert_eq!(list.len(), 3);

    let ids: Vec<&str> = list.iter().map(|(id, _, _, _)| *id).collect();
    assert!(ids.contains(&"telegram"));
    assert!(ids.contains(&"mqtt"));
    assert!(ids.contains(&"serial"));
}

#[test]
fn test_registry_get_port() {
    let mut registry = PortRegistry::new();
    let port = MemoryPort::new("mem1", PortProtocol::Custom("test".into()), PortDirection::Bidirectional);
    registry.register(Box::new(port));
    let got = registry.get("mem1");
    assert!(got.is_some());
    assert_eq!(got.unwrap().id(), "mem1");
    assert!(registry.get("nope").is_none());
}

// ─── FilePort Tests ────────────────────────────────────────────────────────

#[test]
fn test_file_port_basics() {
    let path = "/tmp/test-lau-port-file.txt";
    let _ = std::fs::remove_file(path);
    let mut port = FilePort::new("file1", path, FileFormat::Raw);
    assert_eq!(port.id(), "file1");
    assert!(!port.is_connected());
    port.connect().unwrap();
    assert!(port.is_connected());

    let msg = OutboundMessage::text("hello file");
    port.send(msg).unwrap();

    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("hello file"));

    port.disconnect().unwrap();
    assert!(!port.is_connected());
    let _ = std::fs::remove_file(path);
}

#[test]
fn test_file_port_jsonl_format() {
    let path = "/tmp/test-lau-port-jsonl.txt";
    let _ = std::fs::remove_file(path);
    let mut port = FilePort::new("jsonl1", path, FileFormat::Jsonl);
    port.connect().unwrap();
    port.send(OutboundMessage::text("log entry")).unwrap();
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("\"content\":"));
    assert!(content.contains("log entry"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn test_file_port_cannot_receive() {
    let path = "/tmp/test-lau-port-receive.txt";
    let mut port = FilePort::new("f1", path, FileFormat::Raw);
    port.connect().unwrap();
    let result = port.receive();
    assert!(result.is_err());
    let _ = std::fs::remove_file(path);
}

#[test]
fn test_file_port_not_connected_error() {
    let port = FilePort::new("f1", "/tmp/nonexistent/test.txt", FileFormat::Raw);
    let result = port.send(OutboundMessage::text("fail"));
    assert!(result.is_err());
    match result {
        Err(PortError::NotConnected(_)) => {}
        _ => panic!("expected NotConnected"),
    }
}

// ─── SensorReading Tests ────────────────────────────────────────────────────

#[test]
fn test_sensor_content() {
    let readings = vec![
        SensorReading {
            sensor_id: "temp-1".to_string(),
            value: 22.5,
            unit: "°C".to_string(),
            timestamp: 1700000000,
        },
        SensorReading {
            sensor_id: "humid-1".to_string(),
            value: 55.0,
            unit: "%".to_string(),
            timestamp: 1700000000,
        },
    ];
    let content = MessageContent::Sensor { readings };
    assert!(!content.is_text());
    assert!(!content.is_command());
    if let MessageContent::Sensor { readings } = content {
        assert_eq!(readings.len(), 2);
        assert_eq!(readings[0].value, 22.5);
        assert_eq!(readings[1].unit, "%");
    } else {
        panic!("expected Sensor");
    }
}

// ─── Serialization / Deserialization Tests ──────────────────────────────────

#[test]
fn test_inbound_message_serde() {
    let msg = InboundMessage::text("hello");
    let json = serde_json::to_string(&msg).unwrap();
    let back: InboundMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(back.as_text(), Some("hello"));
}

#[test]
fn test_outbound_message_serde() {
    let msg = OutboundMessage::reply("parent-id", "reply text");
    let json = serde_json::to_string(&msg).unwrap();
    let back: OutboundMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(back.reply_to, Some("parent-id".to_string()));
    assert!(back.content.is_text());
    assert_eq!(back.content.as_text(), Some("reply text"));
}

#[test]
fn test_message_content_serde_roundtrip() {
    let contents = vec![
        MessageContent::text("plain text"),
        MessageContent::Html("<p>html</p>".to_string()),
        MessageContent::Markdown("# md".to_string()),
        MessageContent::Command {
            name: "run".to_string(),
            args: vec!["--flag".to_string()],
        },
        MessageContent::Event {
            event_type: "click".to_string(),
            payload: r#"{"x":10,"y":20}"#.to_string(),
        },
    ];
    for c in contents {
        let json = serde_json::to_string(&c).unwrap();
        let back: MessageContent = serde_json::from_str(&json).unwrap();
        assert_eq!(
            serde_json::to_string(&back).unwrap(),
            json,
            "roundtrip failed for {json}"
        );
    }
}

// ─── PortProtocol Serialization Test ────────────────────────────────────────

#[test]
fn test_protocol_serde_roundtrip() {
    let protos = vec![
        PortProtocol::Telegram,
        PortProtocol::Discord,
        PortProtocol::Mqtt,
        PortProtocol::Custom("MyProto".to_string()),
    ];
    for p in protos {
        let json = serde_json::to_string(&p).unwrap();
        let back: PortProtocol = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }
}

// ─── Stats Tests ────────────────────────────────────────────────────────────

#[test]
fn test_stats_defaults() {
    let stats = PortStats::default();
    assert_eq!(stats.messages_sent, 0);
    assert_eq!(stats.messages_received, 0);
    assert_eq!(stats.bytes_sent, 0);
    assert_eq!(stats.bytes_received, 0);
    assert_eq!(stats.errors, 0);
    assert!(stats.last_activity.is_none());
    assert_eq!(stats.avg_latency_ms, 0.0);
}

#[test]
fn test_stats_serialize() {
    let stats = PortStats {
        messages_sent: 42,
        messages_received: 10,
        bytes_sent: 2048,
        bytes_received: 512,
        errors: 1,
        last_activity: Some(1700000000),
        avg_latency_ms: 150.5,
    };
    let json = serde_json::to_string(&stats).unwrap();
    let back: PortStats = serde_json::from_str(&json).unwrap();
    assert_eq!(back.messages_sent, 42);
    assert_eq!(back.avg_latency_ms, 150.5);
}

// ─── Edge Cases ─────────────────────────────────────────────────────────────

#[test]
fn test_inbound_message_with_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("thread_id".to_string(), "123".to_string());
    metadata.insert("chat_type".to_string(), "group".to_string());
    let msg = InboundMessage {
        id: Uuid::new_v4().to_string(),
        port_id: "tg-1".to_string(),
        sender: Some("user-42".to_string()),
        content: MessageContent::text("hello group"),
        timestamp: 1700000000,
        reply_to: Some("msg-1".to_string()),
        metadata,
    };
    assert_eq!(msg.metadata.get("thread_id").unwrap(), "123");
    assert_eq!(msg.metadata.get("chat_type").unwrap(), "group");
    assert_eq!(msg.sender.as_deref(), Some("user-42"));
}

#[test]
fn test_empty_port_registry_health() {
    let registry = PortRegistry::new();
    let health = registry.health_check();
    assert!(health.is_empty());
}

#[test]
fn test_memory_port_drain_empties_outbox() {
    let mut port = MemoryPort::new("mem1", PortProtocol::Custom("test".into()), PortDirection::Bidirectional);
    port.connect().unwrap();
    port.send(OutboundMessage::text("msg1")).unwrap();
    let first = port.drain_outbox();
    assert_eq!(first.len(), 1);
    let second = port.drain_outbox();
    assert_eq!(second.len(), 0);
}

#[test]
fn test_port_protocol_equality() {
    assert_eq!(PortProtocol::Http, PortProtocol::Http);
    assert_ne!(PortProtocol::Http, PortProtocol::Https);
    assert_eq!(
        PortProtocol::File("a".into()),
        PortProtocol::File("a".into())
    );
    assert_ne!(
        PortProtocol::File("a".into()),
        PortProtocol::File("b".into())
    );
}

#[test]
fn test_port_config_with_all_fields() {
    let config = PortConfig {
        id: "full-config".to_string(),
        protocol: PortProtocol::WebSocket,
        direction: PortDirection::Bidirectional,
        params: [("url".to_string(), "ws://echo".to_string())].into(),
        enabled: true,
        max_rate: Some(100),
        deadband: Some(PortDeadband {
            lower: 0.0,
            upper: 100.0,
            check_interval_ms: 1000,
        }),
    };
    let json = config.to_json();
    let parsed = PortConfig::from_json(&json).unwrap();
    assert_eq!(parsed.id, "full-config");
    assert_eq!(parsed.protocol, PortProtocol::WebSocket);
    assert_eq!(parsed.deadband.unwrap().lower, 0.0);
    assert_eq!(parsed.max_rate, Some(100));
}
