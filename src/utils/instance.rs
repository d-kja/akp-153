use std::{error::Error, fmt::Display, sync::Arc};

use elgato_streamdeck::{info::ImageFormat, list_devices, new_hidapi, StreamDeck};
use image::open;
use tracing::info;

pub struct Instance {
    pub device: StreamDeck,
}

#[non_exhaustive]
#[derive(Debug)]
pub enum InstanceError {
    CreationError(String),
    UnableToConnect,
    PathNotFound,
    Generic(String),
}

impl Display for InstanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use InstanceError::*;
        match self {
            CreationError(msg) => write!(f, "Unable to create instance, reason {}", msg),
            UnableToConnect => write!(f, "Unable to connect to the device"),
            PathNotFound => write!(f, "The path provided wasn't found"),
            Generic(msg) => write!(f, "{msg}"),
        }
    }
}

impl Error for InstanceError {}

unsafe impl std::marker::Sync for Instance {}
unsafe impl std::marker::Send for Instance {}

impl Into<Arc<StreamDeck>> for Instance {
    fn into(self) -> Arc<StreamDeck> {
        Arc::new(self.device)
    }
}

impl Instance {
    pub fn new() -> Result<Self, InstanceError> {
        let hidapi = new_hidapi();

        match hidapi {
            Ok(hid) => {
                let mut devices = list_devices(&hid);
                if devices.is_empty() {
                    return Err(InstanceError::CreationError("no instance found".to_owned()));
                }

                let (kind, serial) = devices.remove(0);
                info!("Found {kind:?}");

                let device = StreamDeck::connect(&hid, kind, &serial);

                if device.is_err() {
                    return Err(InstanceError::UnableToConnect);
                }

                let device = device.unwrap();
                let (serial_number, firmware_version) = (
                    device.serial_number().unwrap(),
                    device.firmware_version().unwrap(),
                );
                info!(
                    "Connected to {:?}, serial number {} and firmware version {}",
                    kind, serial_number, firmware_version
                );

                Ok(Self { device })
            }
            Err(value) => Err(InstanceError::CreationError(value.to_string())),
        }
    }

    pub fn flush(&mut self) -> Result<(), InstanceError> {
        if self.device.is_updated() {
            let result = self.device.flush();
            if let Err(msg) = result {
                return Err(InstanceError::Generic(msg.to_string()));
            }

            result.unwrap();
        }

        Ok(())
    }

    pub fn update_brightness(&mut self, percentage: u8) -> Result<(), InstanceError> {
        let result = self.device.set_brightness(percentage);
        if let Err(message) = result {
            return Err(InstanceError::Generic(message.to_string()));
        }

        result.unwrap();

        Ok(())
    }

    pub fn get_lcd_format(&self) -> Option<ImageFormat> {
        self.device.kind().lcd_image_format()
    }

    pub fn update_background(&mut self, path: &str) -> Result<(), InstanceError> {
        if path.is_empty() {
            return Err(InstanceError::Generic("Invalid path".to_string()));
        }

        let image = open(path);
        if image.is_err() {
            return Err(InstanceError::PathNotFound);
        }

        let image = image.unwrap();

        let response = self.device.set_logo_image(image.clone());
        if let Err(message) = response {
            return Err(InstanceError::Generic(message.to_string()));
        }

        response.unwrap();

        Ok(())
    }
}
