use rppal::gpio::Gpio;
use rumqttc::{Client, Event, MqttOptions, Packet, QoS};

use std::error::Error;
use std::time::Duration;

// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
const GPIO_I1: u8 = 5; // EVU lock
const GPIO_I3: u8 = 22; // PV mode

fn main() -> Result<(), Box<dyn Error>> {
    let mut i1 = Gpio::new().unwrap().get(GPIO_I1).unwrap().into_output();
    let mut i3 = Gpio::new().unwrap().get(GPIO_I3).unwrap().into_output();

    let mut mqttoptions = MqttOptions::new("rs-rpi-mqtt-gpio", "192.168.9.2", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_clean_session(false);

    let (mut client, mut connection) = Client::new(mqttoptions, 10);
    client
        .subscribe("heatpump/output/#", QoS::AtLeastOnce)
        .unwrap();

    // Iterate to poll the eventloop for connection progress
    for (_, notification) in connection.iter().enumerate() {
        // Ignore all messages but published incoming packets
        let notifi = match notification {
            Ok(n) => n,
            _ => continue,
        };

        let noti = match notifi {
            Event::Incoming(n) => n,
            _ => continue,
        };

        let n = match noti {
            Packet::Publish(n) => n,
            _ => continue,
        };

        println!("Notification = {:?} {:?}", n.topic, n.payload);

        if n.payload == "ON" {
            match n.topic.as_str() {
                "heatpump/output/i1/set" => {
                    println!("i1 high");
                    i1.set_high();
                    client.publish("heatpump/output/i1", QoS::AtLeastOnce, false, "ON")
                }
                "heatpump/output/i3/set" => {
                    println!("i3 high");
                    i3.set_high();
                    client.publish("heatpump/output/i3", QoS::AtLeastOnce, false, "ON")
                }
                _ => continue,
            };
        } else if n.payload == "OFF" {
            match n.topic.as_str() {
                "heatpump/output/i1/set" => {
                    println!("i1 low");
                    i1.set_low();
                    client.publish("heatpump/output/i1", QoS::AtLeastOnce, false, "OFF")
                }
                "heatpump/output/i3/set" => {
                    println!("i3 low");
                    i3.set_low();
                    client.publish("heatpump/output/i3", QoS::AtLeastOnce, false, "OFF")
                }
                _ => continue,
            };
        }
    }

    Ok(())
}
