mod http;
mod utils;

use std::sync::Arc;
use std::time::Duration;

use elgato_streamdeck::{DeviceStateUpdate, StreamDeck};
use tracing::{error, info, warn};
use utils::instance::Instance;

fn main() {
    // Setup logs
    std::env::set_var("RUST_LOG", "trace");
    tracing_subscriber::fmt::init();

    let instance = Instance::new();
    if let Err(message) = &instance {
        error!("{}", message);
    }

    let device: Arc<StreamDeck> = instance.unwrap().into();
    let reader = device.get_reader();

    let mut key_count: [u8; 2] = [0, 0];

    'wrap: loop {
        let updates = match reader.read(Some(Duration::from_secs_f64(200.0))) {
            Ok(updates) => updates,
            Err(_) => {
                info!("Nothing was found");
                break;
            }
        };

        for update in updates {
            match update {
                DeviceStateUpdate::ButtonDown(key) => {
                    if key_count[0].eq(&key) {
                        key_count[1] += 1;

                        if key_count[1] >= 6 {
                            warn!("Stopping program");
                            break 'wrap;
                        }
                    } else {
                        key_count = [key, 0];
                    }

                    info!("Button {} down, count {}", key, key_count[1]);
                },

                _ => continue,
            }
        }
    }

    drop(reader);
    device.shutdown().ok();
}
