use std::env;
use std::error::Error;
use std::fs::File;
use std::time::Duration;

use bytes::Bytes;
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use rumqttc::{Client, ClientError, Connection, Event, MqttOptions, Packet, QoS, SubscribeFilter};

mod config;

struct Mqtt<'a> {
    name: &'a str,
    host: &'a str,
    port: u16,
    qos: QoS,
    cap: usize,
    keep_alive: Duration,
    clean_session: bool,
    subscribe: Vec<&'a str>,
    mqtt_client: MqttClient,
}

// This sub-struct is required, because we can't borrow Option<> properly otherwise
struct MqttClient {
    client: Option<Client>,
    connection: Option<Connection>,
}

impl MqttClient {
    fn new() -> Self {
        MqttClient {
            client: None,
            connection: None,
        }
    }
}

impl<'a> Mqtt<'a> {
    fn new(name: &'a str, host: &'a str, port: u16) -> Self {
        Mqtt {
            name,
            host,
            port,
            qos: QoS::AtLeastOnce,
            keep_alive: Duration::from_secs(5),
            cap: 10,
            clean_session: false,
            subscribe: vec!["#"],
            mqtt_client: MqttClient::new(),
        }
    }

    fn connect(&mut self) -> Result<(), ClientError> {
        let mut mqtt_options = MqttOptions::new(self.name, self.host, self.port);
        mqtt_options.set_keep_alive(self.keep_alive);
        mqtt_options.set_clean_session(self.clean_session);

        let (mut client, connection) = Client::new(mqtt_options, self.cap);

        println!("Subscribing to topics: {:?}", self.subscribe);
        client.subscribe_many(
            self.subscribe
                .iter()
                .map(|topic| SubscribeFilter::new(topic.to_string(), self.qos)),
        )?;
        self.mqtt_client.client = Some(client);
        self.mqtt_client.connection = Some(connection);
        Ok(())
    }

    fn event_loop(&mut self, pins: &[Pin]) -> Result<(), Box<dyn Error>> {
        // Iterate to poll the eventloop for connection progress
        for (_i, notification) in self
            .mqtt_client
            .connection
            .as_mut()
            .expect("MqttClient::connection not set")
            .iter()
            .enumerate()
        {
            let Ok(Event::Incoming(Packet::Publish(n))) = notification else { continue };

            let client = self
                .mqtt_client
                .client
                .as_mut()
                .expect("MqttClient::client not set");

            // Check whether incoming topic matches, if so set pin
            for p in pins.iter() {
                if n.topic == p.mqtt_topic_set {
                    p.set_and_publish_state(client, n.payload.to_owned())?;
                    continue;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct Pin<'a> {
    line_handle: LineHandle,
    // TODO: Infer topics from name and prefix like mqtt-io?
    mqtt_topic: &'a str,
    mqtt_topic_set: &'a str,
    mqtt_state_on: &'a str,
    mqtt_state_off: &'a str,
    qos: QoS,
    retain: bool,
}

impl<'a> Pin<'a> {
    fn new(line_handle: LineHandle) -> Self {
        Pin {
            line_handle,
            mqtt_topic: "gpio",
            mqtt_topic_set: "gpio/set",
            mqtt_state_on: "ON",
            mqtt_state_off: "OFF",
            qos: QoS::AtLeastOnce,
            retain: false,
        }
    }

    fn publish_state(&self, mqtt_client: &mut Client) -> Result<(), gpio_cdev::Error> {
        let value = self.line_handle.get_value();

        let state = match value {
            Ok(1) => self.mqtt_state_on,
            Ok(0) => self.mqtt_state_off,
            _ => panic!("GPIO pin returned neither 0 nor 1, this should never happen!"),
        };

        mqtt_client
            .publish(self.mqtt_topic, self.qos, self.retain, state.to_owned())
            .unwrap_or_else(|e| println!("Error publishing: {e}"));
        Ok(())
    }

    fn set_and_publish_state(
        &self,
        mqtt_client: &mut Client,
        payload: Bytes,
    ) -> Result<(), gpio_cdev::Error> {
        if payload == self.mqtt_state_on {
            println!("Setting {} = {}", self.mqtt_topic, self.mqtt_state_on);
            self.line_handle.set_value(1)?;
            self.publish_state(mqtt_client)?;
        } else if payload == self.mqtt_state_off {
            println!("Setting {} = {}", self.mqtt_topic, self.mqtt_state_off);
            self.line_handle.set_value(0)?;
            self.publish_state(mqtt_client)?;
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // First argument is config file
    let config_file = match env::args().nth(1) {
        Some(f) => f,
        None => {
            eprintln!("Usage: rpi-mqtt-gpio <config.yaml>");
            std::process::exit(1);
        }
    };

    let reader = File::open(config_file)?;
    let conf: config::Config = serde_yaml::from_reader(reader)?;

    let mut mqtt = Mqtt::new(&conf.mqtt.name, &conf.mqtt.host, conf.mqtt.port);
    mqtt.qos = match conf.mqtt.qos {
        0 => QoS::AtMostOnce,
        1 => QoS::AtLeastOnce,
        2 => QoS::ExactlyOnce,
        _ => {
            eprintln!("Invalid QoS specified, must be one of 0,1,2");
            std::process::exit(1);
        }
    };
    mqtt.keep_alive = Duration::from_secs(conf.mqtt.keep_alive);
    mqtt.cap = conf.mqtt.cap;
    mqtt.clean_session = conf.mqtt.clean_session;
    mqtt.subscribe = conf
        .digital_outputs
        .iter()
        .map(|s| s.mqtt_topic_set.as_str())
        .collect();
    mqtt.connect()?;

    let mut chip = Chip::new(&conf.chip)?;
    let mut pins: Vec<Pin> = Vec::with_capacity(2);

    for output in conf.digital_outputs.iter() {
        // NOTE: OUTPUT lines can also handle get_value() requests
        let handle =
            chip.get_line(output.gpio)?
                .request(LineRequestFlags::OUTPUT, 0, "write-gpio")?;

        let mut i = Pin::new(handle);
        i.mqtt_topic = &output.mqtt_topic;
        i.mqtt_topic_set = &output.mqtt_topic_set;
        i.retain = conf.mqtt.retain;
        println!(
            "Registered GPIO pin {} (via {:?})",
            output.gpio,
            i.line_handle.line().chip().path()
        );
        pins.push(i);
    }

    // Get GPIO state and broadcast
    let client = mqtt
        .mqtt_client
        .client
        .as_mut()
        .expect("MqttClient::client not set");

    for pin in &pins {
        pin.publish_state(client)?;
    }

    mqtt.event_loop(&pins)?;
    Ok(())
}
