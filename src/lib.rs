use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::HashMap;
use uuid::Uuid;

// ─── Protocols ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortProtocol {
    Telegram,
    Discord,
    Slack,
    WebSocket,
    Http,
    Https,
    Mqtt,
    Amqp,
    Serial,
    Gpio,
    I2c,
    Spi,
    StdinStdout,
    File(String),
    Custom(String),
}

impl PortProtocol {
    pub fn label(&self) -> &str {
        match self {
            PortProtocol::Telegram => "Telegram",
            PortProtocol::Discord => "Discord",
            PortProtocol::Slack => "Slack",
            PortProtocol::WebSocket => "WebSocket",
            PortProtocol::Http => "HTTP",
            PortProtocol::Https => "HTTPS",
            PortProtocol::Mqtt => "MQTT",
            PortProtocol::Amqp => "AMQP",
            PortProtocol::Serial => "Serial",
            PortProtocol::Gpio => "GPIO",
            PortProtocol::I2c => "I2C",
            PortProtocol::Spi => "SPI",
            PortProtocol::StdinStdout => "stdin/stdout",
            PortProtocol::File(_) => "File",
            PortProtocol::Custom(name) => name.as_str(),
        }
    }

    pub fn is_network(&self) -> bool {
        matches!(
            self,
            PortProtocol::Telegram
                | PortProtocol::Discord
                | PortProtocol::Slack
                | PortProtocol::WebSocket
                | PortProtocol::Http
                | PortProtocol::Https
                | PortProtocol::Mqtt
                | PortProtocol::Amqp
        )
    }

    pub fn is_hardware(&self) -> bool {
        matches!(
            self,
            PortProtocol::Serial | PortProtocol::Gpio | PortProtocol::I2c | PortProtocol::Spi
        )
    }

    pub fn is_filesystem(&self) -> bool {
        matches!(self, PortProtocol::File(_))
    }
}

// ─── Direction ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortDirection {
    Inbound,
    Outbound,
    Bidirectional,
}

impl PortDirection {
    pub fn can_send(&self) -> bool {
        matches!(self, PortDirection::Outbound | PortDirection::Bidirectional)
    }

    pub fn can_receive(&self) -> bool {
        matches!(self, PortDirection::Inbound | PortDirection::Bidirectional)
    }
}

// ─── Message content ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    pub sensor_id: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    Html(String),
    Markdown(String),
    Image {
        data: Vec<u8>,
        mime: String,
        caption: Option<String>,
    },
    Audio {
        data: Vec<u8>,
        mime: String,
        duration_ms: Option<u32>,
    },
    File {
        data: Vec<u8>,
        mime: String,
        name: String,
    },
    Command {
        name: String,
        args: Vec<String>,
    },
    Event {
        event_type: String,
        payload: String,
    },
    Sensor {
        readings: Vec<SensorReading>,
    },
}

impl MessageContent {
    pub fn text(content: &str) -> Self {
        MessageContent::Text(content.to_string())
    }

    pub fn command(name: &str, args: &[&str]) -> Self {
        MessageContent::Command {
            name: name.to_string(),
            args: args.iter().map(|a| a.to_string()).collect(),
        }
    }

    pub fn is_text(&self) -> bool {
        matches!(self, MessageContent::Text(_))
    }

    pub fn is_command(&self) -> bool {
        matches!(self, MessageContent::Command { .. })
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            MessageContent::Text(t) => Some(t.as_str()),
            _ => None,
        }
    }
}

// ─── Messages ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundMessage {
    pub id: String,
    pub port_id: String,
    pub sender: Option<String>,
    pub content: MessageContent,
    pub timestamp: i64,
    pub reply_to: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl InboundMessage {
    pub fn text(content: &str) -> Self {
        InboundMessage {
            id: Uuid::new_v4().to_string(),
            port_id: String::new(),
            sender: None,
            content: MessageContent::text(content),
            timestamp: chrono::Utc::now().timestamp(),
            reply_to: None,
            metadata: HashMap::new(),
        }
    }

    pub fn command(name: &str, args: &[&str]) -> Self {
        InboundMessage {
            id: Uuid::new_v4().to_string(),
            port_id: String::new(),
            sender: None,
            content: MessageContent::command(name, args),
            timestamp: chrono::Utc::now().timestamp(),
            reply_to: None,
            metadata: HashMap::new(),
        }
    }

    pub fn is_text(&self) -> bool {
        self.content.is_text()
    }

    pub fn is_command(&self) -> bool {
        self.content.is_command()
    }

    pub fn as_text(&self) -> Option<&str> {
        self.content.as_text()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundMessage {
    pub id: String,
    pub port_id: String,
    pub target: Option<String>,
    pub content: MessageContent,
    pub reply_to: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl OutboundMessage {
    pub fn text(content: &str) -> Self {
        OutboundMessage {
            id: Uuid::new_v4().to_string(),
            port_id: String::new(),
            target: None,
            content: MessageContent::text(content),
            reply_to: None,
            metadata: HashMap::new(),
        }
    }

    pub fn reply(reply_to: &str, content: &str) -> Self {
        OutboundMessage {
            id: Uuid::new_v4().to_string(),
            port_id: String::new(),
            target: None,
            content: MessageContent::text(content),
            reply_to: Some(reply_to.to_string()),
            metadata: HashMap::new(),
        }
    }
}

// ─── Deadband ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDeadband {
    pub lower: f64,
    pub upper: f64,
    pub check_interval_ms: u64,
}

// ─── Port Config ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConfig {
    pub id: String,
    pub protocol: PortProtocol,
    pub direction: PortDirection,
    pub params: HashMap<String, String>,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_rate: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadband: Option<PortDeadband>,
}

impl PortConfig {
    pub fn from_json(json: &str) -> Result<Self, PortError> {
        serde_json::from_str(json).map_err(|e| PortError::ConfigInvalid(e.to_string()))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

// ─── Port Stats ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PortStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub errors: u64,
    pub last_activity: Option<i64>,
    pub avg_latency_ms: f64,
}

// ─── Port Error ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum PortError {
    NotConnected(String),
    SendFailed(String),
    ReceiveFailed(String),
    ConfigInvalid(String),
    RateLimited { retry_after_ms: u64 },
    AuthFailed(String),
    Timeout(u64),
    ProtocolError(String),
    ConnectFailed(String),
}

impl std::fmt::Display for PortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortError::NotConnected(s) => write!(f, "not connected: {s}"),
            PortError::SendFailed(s) => write!(f, "send failed: {s}"),
            PortError::ReceiveFailed(s) => write!(f, "receive failed: {s}"),
            PortError::ConfigInvalid(s) => write!(f, "invalid config: {s}"),
            PortError::RateLimited { retry_after_ms } => {
                write!(f, "rate limited, retry after {retry_after_ms}ms")
            }
            PortError::AuthFailed(s) => write!(f, "auth failed: {s}"),
            PortError::Timeout(ms) => write!(f, "timeout after {ms}ms"),
            PortError::ProtocolError(s) => write!(f, "protocol error: {s}"),
            PortError::ConnectFailed(s) => write!(f, "connect failed: {s}"),
        }
    }
}

impl std::error::Error for PortError {}

// ─── Port Trait ─────────────────────────────────────────────────────────────

pub trait Port: Send + Sync {
    fn id(&self) -> &str;
    fn protocol(&self) -> PortProtocol;
    fn direction(&self) -> PortDirection;
    fn send(&self, message: OutboundMessage) -> Result<(), PortError>;
    fn receive(&self) -> Result<Vec<InboundMessage>, PortError>;
    fn is_connected(&self) -> bool;
    fn connect(&mut self) -> Result<(), PortError>;
    fn disconnect(&mut self) -> Result<(), PortError>;
    fn stats(&self) -> PortStats;
}

// ─── StdioPort ──────────────────────────────────────────────────────────────

pub struct StdioPort {
    id: String,
    connected: Mutex<bool>,
    direction: PortDirection,
    stats: Mutex<PortStats>,
}

impl StdioPort {
    pub fn new(id: &str) -> Self {
        StdioPort {
            id: id.to_string(),
            connected: Mutex::new(false),
            direction: PortDirection::Bidirectional,
            stats: Mutex::new(PortStats::default()),
        }
    }
}

impl Port for StdioPort {
    fn id(&self) -> &str {
        &self.id
    }

    fn protocol(&self) -> PortProtocol {
        PortProtocol::StdinStdout
    }

    fn direction(&self) -> PortDirection {
        self.direction
    }

    fn send(&self, message: OutboundMessage) -> Result<(), PortError> {
        if !*self.connected.lock().unwrap() {
            return Err(PortError::NotConnected(
                "stdio port is not connected".to_string(),
            ));
        }
        let text = match &message.content {
            MessageContent::Text(t) => t.clone(),
            MessageContent::Html(h) => h.clone(),
            MessageContent::Markdown(m) => m.clone(),
            _ => return Err(PortError::SendFailed("unsupported content type".into())),
        };
        {
            let mut stats = self.stats.lock().unwrap();
            stats.messages_sent += 1;
            stats.bytes_sent += text.len() as u64;
            stats.last_activity = Some(chrono::Utc::now().timestamp());
        }
        println!("{text}");
        Ok(())
    }

    fn receive(&self) -> Result<Vec<InboundMessage>, PortError> {
        Err(PortError::NotConnected(
            "stdio receive not implemented; use stdin reader externally".to_string(),
        ))
    }

    fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }

    fn connect(&mut self) -> Result<(), PortError> {
        *self.connected.lock().unwrap() = true;
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), PortError> {
        *self.connected.lock().unwrap() = false;
        Ok(())
    }

    fn stats(&self) -> PortStats {
        self.stats.lock().unwrap().clone()
    }
}

// ─── MemoryPort ─────────────────────────────────────────────────────────────

pub struct MemoryPort {
    id: String,
    protocol: PortProtocol,
    direction: PortDirection,
    inbox: Mutex<Vec<InboundMessage>>,
    outbox: Mutex<Vec<OutboundMessage>>,
    connected: Mutex<bool>,
    stats: Mutex<PortStats>,
}

impl MemoryPort {
    pub fn new(id: &str, protocol: PortProtocol, direction: PortDirection) -> Self {
        MemoryPort {
            id: id.to_string(),
            protocol,
            direction,
            inbox: Mutex::new(Vec::new()),
            outbox: Mutex::new(Vec::new()),
            connected: Mutex::new(false),
            stats: Mutex::new(PortStats::default()),
        }
    }

    pub fn inject(&self, msg: InboundMessage) {
        {
            let mut stats = self.stats.lock().unwrap();
            stats.messages_received += 1;
            stats.bytes_received += content_bytes(&msg.content) as u64;
            stats.last_activity = Some(chrono::Utc::now().timestamp());
        }
        self.inbox.lock().unwrap().push(msg);
    }

    pub fn drain_outbox(&self) -> Vec<OutboundMessage> {
        self.outbox.lock().unwrap().drain(..).collect()
    }
}

impl Port for MemoryPort {
    fn id(&self) -> &str {
        &self.id
    }

    fn protocol(&self) -> PortProtocol {
        self.protocol.clone()
    }

    fn direction(&self) -> PortDirection {
        self.direction
    }

    fn send(&self, message: OutboundMessage) -> Result<(), PortError> {
        if !*self.connected.lock().unwrap() {
            return Err(PortError::NotConnected(format!(
                "memory port '{}' not connected",
                self.id
            )));
        }
        if !self.direction.can_send() {
            return Err(PortError::SendFailed(format!(
                "port '{}' is {:?}, cannot send",
                self.id, self.direction
            )));
        }
        {
            let mut stats = self.stats.lock().unwrap();
            stats.messages_sent += 1;
            stats.bytes_sent += content_bytes(&message.content) as u64;
            stats.last_activity = Some(chrono::Utc::now().timestamp());
        }
        self.outbox.lock().unwrap().push(message);
        Ok(())
    }

    fn receive(&self) -> Result<Vec<InboundMessage>, PortError> {
        if !*self.connected.lock().unwrap() {
            return Err(PortError::NotConnected(format!(
                "memory port '{}' not connected",
                self.id
            )));
        }
        if !self.direction.can_receive() {
            return Err(PortError::ReceiveFailed(format!(
                "port '{}' is {:?}, cannot receive",
                self.id, self.direction
            )));
        }
        Ok(self.inbox.lock().unwrap().drain(..).collect())
    }

    fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }

    fn connect(&mut self) -> Result<(), PortError> {
        *self.connected.lock().unwrap() = true;
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), PortError> {
        *self.connected.lock().unwrap() = false;
        Ok(())
    }

    fn stats(&self) -> PortStats {
        self.stats.lock().unwrap().clone()
    }
}

// ─── FilePort ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileFormat {
    Jsonl,
    Csv,
    Raw,
}

pub struct FilePort {
    id: String,
    path: String,
    format: FileFormat,
    connected: Mutex<bool>,
    direction: PortDirection,
    stats: Mutex<PortStats>,
}

impl FilePort {
    pub fn new(id: &str, path: &str, format: FileFormat) -> Self {
        FilePort {
            id: id.to_string(),
            path: path.to_string(),
            format,
            connected: Mutex::new(false),
            direction: PortDirection::Outbound,
            stats: Mutex::new(PortStats::default()),
        }
    }
}

impl Port for FilePort {
    fn id(&self) -> &str {
        &self.id
    }

    fn protocol(&self) -> PortProtocol {
        PortProtocol::File(self.path.clone())
    }

    fn direction(&self) -> PortDirection {
        self.direction
    }

    fn send(&self, message: OutboundMessage) -> Result<(), PortError> {
        use std::io::Write;

        if !*self.connected.lock().unwrap() {
            return Err(PortError::NotConnected(format!(
                "file port '{}' not connected",
                self.id
            )));
        }

        let line = match self.format {
            FileFormat::Jsonl => serde_json::to_string(&message)
                .map_err(|e| PortError::SendFailed(e.to_string()))?,
            FileFormat::Csv => {
                format!(
                    "{},{},{}\n",
                    message.id,
                    message.target.unwrap_or_default(),
                    text_preview(&message.content)
                )
            }
            FileFormat::Raw => match &message.content {
                MessageContent::Text(t) => t.clone(),
                MessageContent::Html(h) => h.clone(),
                MessageContent::Markdown(m) => m.clone(),
                _ => return Err(PortError::SendFailed("unsupported content for raw".into())),
            },
        };

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| PortError::SendFailed(e.to_string()))?;

        writeln!(file, "{line}")
            .map_err(|e| PortError::SendFailed(e.to_string()))?;

        {
            let mut stats = self.stats.lock().unwrap();
            stats.messages_sent += 1;
            stats.bytes_sent += line.len() as u64 + 1;
            stats.last_activity = Some(chrono::Utc::now().timestamp());
        }

        Ok(())
    }

    fn receive(&self) -> Result<Vec<InboundMessage>, PortError> {
        Err(PortError::NotConnected("file ports are write-only".into()))
    }

    fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }

    fn connect(&mut self) -> Result<(), PortError> {
        let parent = std::path::Path::new(&self.path).parent().unwrap_or(std::path::Path::new("."));
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| PortError::ConnectFailed(e.to_string()))?;
        }
        *self.connected.lock().unwrap() = true;
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), PortError> {
        *self.connected.lock().unwrap() = false;
        Ok(())
    }

    fn stats(&self) -> PortStats {
        self.stats.lock().unwrap().clone()
    }
}

// ─── PortRegistry ───────────────────────────────────────────────────────────

pub struct PortRegistry {
    ports: HashMap<String, Box<dyn Port>>,
    configs: HashMap<String, PortConfig>,
}

impl PortRegistry {
    pub fn new() -> Self {
        PortRegistry {
            ports: HashMap::new(),
            configs: HashMap::new(),
        }
    }

    pub fn register(&mut self, port: Box<dyn Port>) {
        let id = port.id().to_string();
        self.ports.insert(id, port);
    }

    pub fn register_with_config(&mut self, port: Box<dyn Port>, config: PortConfig) {
        let id = port.id().to_string();
        self.configs.insert(id.clone(), config);
        self.ports.insert(id, port);
    }

    pub fn unregister(&mut self, port_id: &str) -> Result<Box<dyn Port>, PortError> {
        self.ports
            .remove(port_id)
            .ok_or_else(|| PortError::NotConnected(format!("port '{port_id}' not found")))
    }

    pub fn send(&self, port_id: &str, message: OutboundMessage) -> Result<(), PortError> {
        let port = self
            .ports
            .get(port_id)
            .ok_or_else(|| PortError::NotConnected(format!("port '{port_id}' not found")))?;
        port.send(message)
    }

    pub fn receive_all(&self) -> Vec<(String, Result<Vec<InboundMessage>, PortError>)> {
        self.ports
            .iter()
            .map(|(id, port)| {
                let result = match port.direction().can_receive() {
                    true => port.receive(),
                    false => Err(PortError::NotConnected(format!(
                        "port '{id}' cannot receive"
                    ))),
                };
                (id.clone(), result)
            })
            .collect()
    }

    pub fn get(&self, port_id: &str) -> Option<&dyn Port> {
        self.ports.get(port_id).map(|p| p.as_ref())
    }

    pub fn get_mut(&mut self, port_id: &str) -> Option<&mut Box<dyn Port>> {
        self.ports.get_mut(port_id)
    }

    pub fn list(&self) -> Vec<(&str, PortProtocol, PortDirection, bool)> {
        self.ports
            .iter()
            .map(|(id, p)| (id.as_str(), p.protocol(), p.direction(), p.is_connected()))
            .collect()
    }

    pub fn connect_all(&mut self) -> Vec<(String, Result<(), PortError>)> {
        self.ports
            .iter_mut()
            .map(|(id, port)| (id.clone(), port.connect()))
            .collect()
    }

    pub fn disconnect_all(&mut self) {
        for port in self.ports.values_mut() {
            let _ = port.disconnect();
        }
    }

    pub fn health_check(&self) -> HashMap<String, bool> {
        self.ports
            .iter()
            .map(|(id, port)| (id.clone(), port.is_connected()))
            .collect()
    }
}

impl Default for PortRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ─── helpers ────────────────────────────────────────────────────────────────

fn content_bytes(content: &MessageContent) -> usize {
    match content {
        MessageContent::Text(t) => t.len(),
        MessageContent::Html(h) => h.len(),
        MessageContent::Markdown(m) => m.len(),
        MessageContent::Image { data, .. } => data.len(),
        MessageContent::Audio { data, .. } => data.len(),
        MessageContent::File { data, .. } => data.len(),
        MessageContent::Command { name, args } => {
            name.len() + args.iter().map(|a| a.len()).sum::<usize>()
        }
        MessageContent::Event {
            event_type,
            payload,
        } => event_type.len() + payload.len(),
        MessageContent::Sensor { readings } => readings.len() * 32,
    }
}

fn text_preview(content: &MessageContent) -> String {
    match content {
        MessageContent::Text(t) => t.chars().take(80).collect(),
        MessageContent::Html(h) => h.chars().take(80).collect(),
        MessageContent::Markdown(m) => m.chars().take(80).collect(),
        MessageContent::Image { caption, .. } => {
            format!("[image: {}]", caption.as_deref().unwrap_or("no caption"))
        }
        MessageContent::Audio { .. } => "[audio]".to_string(),
        MessageContent::File { name, .. } => format!("[file: {name}]"),
        MessageContent::Command { name, args } => {
            format!("cmd /{name} {:?}", args.join(" "))
        }
        MessageContent::Event { event_type, .. } => format!("[event: {event_type}]"),
        MessageContent::Sensor { readings } => {
            format!("[sensor: {} readings]", readings.len())
        }
    }
}
