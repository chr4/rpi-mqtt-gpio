use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_chip")]
    pub chip: String,
    pub mqtt: MqttConfig,
    pub digital_outputs: Vec<DigitalOutput>,
}

impl Config {
    fn default_chip() -> String {
        "/dev/gpiochip0".to_string()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MqttConfig {
    #[serde(default = "MqttConfig::default_name")]
    pub name: String,
    #[serde(default = "MqttConfig::default_host")]
    pub host: String,
    #[serde(default = "MqttConfig::default_port")]
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default = "MqttConfig::default_qos")]
    pub qos: u8,
    #[serde(default = "MqttConfig::default_keep_alive")]
    pub keep_alive: u64,
    #[serde(default = "MqttConfig::default_clean_session")]
    pub clean_session: bool,
    #[serde(default = "MqttConfig::default_retain")]
    pub retain: bool,
    #[serde(default = "MqttConfig::default_cap")]
    pub cap: usize,
    #[serde(default = "MqttConfig::default_availability_topic")]
    pub availability_topic: String,
    #[serde(default = "MqttConfig::default_payload_available")]
    pub payload_available: String,
    #[serde(default = "MqttConfig::default_payload_not_available")]
    pub payload_not_available: String,
}

impl MqttConfig {
    fn default_name() -> String {
        "rpi-mqtt-gpio".to_string()
    }
    fn default_host() -> String {
        "localhost".to_string()
    }
    fn default_port() -> u16 {
        1883
    }
    fn default_qos() -> u8 {
        1
    }
    fn default_keep_alive() -> u64 {
        5
    }
    fn default_clean_session() -> bool {
        false
    }
    fn default_retain() -> bool {
        false
    }
    fn default_cap() -> usize {
        10
    }
    fn default_availability_topic() -> String {
        "rpi-mqtt-gpio/status".to_string()
    }
    fn default_payload_available() -> String {
        "online".to_string()
    }
    fn default_payload_not_available() -> String {
        "offline".to_string()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DigitalOutput {
    #[serde(default = "DigitalOutput::default_name")]
    pub name: String,
    pub gpio: u32,
    pub mqtt_topic: String,
    pub mqtt_topic_set: String,
    #[serde(default = "DigitalOutput::default_mqtt_state_high")]
    pub mqtt_state_high: String,
    #[serde(default = "DigitalOutput::default_mqtt_state_low")]
    pub mqtt_state_low: String,
    #[serde(default = "DigitalOutput::default_initial_state")]
    pub initial_state: String,
}

impl DigitalOutput {
    fn default_name() -> String {
        "gpio".to_string()
    }
    fn default_mqtt_state_high() -> String {
        "ON".to_string()
    }
    fn default_mqtt_state_low() -> String {
        "OFF".to_string()
    }
    fn default_initial_state() -> String {
        "OFF".to_string()
    }
}
