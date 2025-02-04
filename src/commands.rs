// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::error::Error;
use crate::state::{ReadData, SerialportInfo, SerialportState};
use serialport::{DataBits, FlowControl, Parity, SerialPortType, StopBits};
use std::collections::HashMap;
use std::io::ErrorKind;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager, Runtime, State, Window};

const UNKNOWN: &str = "Unknown";
const USB: &str = "USB";
const BLUETOOTH: &str = "Bluetooth";
const PCI: &str = "PCI";

/// `get_worksheet` gets the file sheet instance according to `path` and `sheet_name`.
fn get_serialport<T, F: FnOnce(&mut SerialportInfo) -> Result<T, Error>>(
    state: State<'_, SerialportState>,
    path: String,
    f: F,
) -> Result<T, Error> {
    match state.serialports.lock() {
        Ok(mut map) => match map.get_mut(&path) {
            Some(serialport_info) => f(serialport_info),
            None => Err(Error::String("serial port not found".to_string())),
        },
        Err(error) => Err(Error::String(format!(
            "Failed to acquire file lock! {}",
            error
        ))),
    }
}

/// `get_worksheet` gets the file sheet instance according to `path` and `sheet_name`.
// fn try_get_serialport<T, F: FnOnce(&mut SerialportInfo) -> Result<T, Error>>(
//     state: Arc<std::sync::Mutex<HashMap<std::string::String, SerialportInfo>>>,
//     path: String,
//     f: F,
// ) -> Result<T, Error> {
//     match state.try_lock() {
//         Ok(mut map) => match map.get_mut(&path) {
//             Some(serialport_info) => return f(serialport_info),
//             None => {
//                 return Err(Error::String(format!("{} serial port not found", &path)));
//             }
//         },
//         Err(error) => return Err(Error::String(format!("Failed to acquire file lock! {} ", error))),
//     }
// }

fn get_data_bits(value: Option<usize>) -> DataBits {
    match value {
        Some(value) => match value {
            5 => DataBits::Five,
            6 => DataBits::Six,
            7 => DataBits::Seven,
            8 => DataBits::Eight,
            _ => DataBits::Eight,
        },
        None => DataBits::Eight,
    }
}

fn get_flow_control(value: Option<String>) -> FlowControl {
    match value {
        Some(value) => match value.as_str() {
            "Software" => FlowControl::Software,
            "Hardware" => FlowControl::Hardware,
            _ => FlowControl::None,
        },
        None => FlowControl::None,
    }
}

fn get_parity(value: Option<String>) -> Parity {
    match value {
        Some(value) => match value.as_str() {
            "Odd" => Parity::Odd,
            "Even" => Parity::Even,
            _ => Parity::None,
        },
        None => Parity::None,
    }
}

fn get_stop_bits(value: Option<usize>) -> StopBits {
    match value {
        Some(value) => match value {
            1 => StopBits::One,
            2 => StopBits::Two,
            _ => StopBits::Two,
        },
        None => StopBits::Two,
    }
}

fn get_port_info(port: SerialPortType) -> HashMap<String, String> {
    let mut port_info: HashMap<String, String> = HashMap::new();
    port_info.insert("type".to_string(), UNKNOWN.to_string());
    port_info.insert("vid".to_string(), UNKNOWN.to_string());
    port_info.insert("pid".to_string(), UNKNOWN.to_string());
    port_info.insert("serial_number".to_string(), UNKNOWN.to_string());
    port_info.insert("manufacturer".to_string(), UNKNOWN.to_string());
    port_info.insert("product".to_string(), UNKNOWN.to_string());

    match port {
        SerialPortType::UsbPort(info) => {
            port_info.insert("type".to_string(), USB.to_string());
            port_info.insert("vid".to_string(), info.vid.to_string());
            port_info.insert("pid".to_string(), info.pid.to_string());
            port_info.insert(
                "serial_number".to_string(),
                info.serial_number.unwrap_or_else(|| UNKNOWN.to_string()),
            );
            port_info.insert(
                "manufacturer".to_string(),
                info.manufacturer.unwrap_or_else(|| UNKNOWN.to_string()),
            );
            port_info.insert(
                "product".to_string(),
                info.product.unwrap_or_else(|| UNKNOWN.to_string()),
            );
        }
        SerialPortType::BluetoothPort => {
            port_info.insert("type".to_string(), BLUETOOTH.to_string());
        }
        SerialPortType::PciPort => {
            port_info.insert("type".to_string(), PCI.to_string());
        }
        SerialPortType::Unknown => {
            port_info.insert("type".to_string(), UNKNOWN.to_string());
        }
    }

    port_info
}

/// `available_ports` get serial port list
#[tauri::command]
pub fn available_ports() -> HashMap<String, HashMap<String, String>> {
    let mut list = match serialport::available_ports() {
        Ok(list) => list,
        Err(_) => vec![],
    };
    list.retain(|port| matches!(port.port_type, serialport::SerialPortType::UsbPort(_)));
    list.sort_by(|a, b| a.port_name.cmp(&b.port_name));

    let mut result_list: HashMap<String, HashMap<String, String>> = HashMap::new();

    for p in list {
        result_list.insert(p.port_name, get_port_info(p.port_type));
    }

    println!("Serial port list: {:?}", &result_list);

    result_list
}

/// `cacel_read` cancel serial port data reading
#[tauri::command]
pub async fn cancel_read<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    state: State<'_, SerialportState>,
    path: String,
) -> Result<(), Error> {
    get_serialport(state, path.clone(), |serialport_info| {
        match &serialport_info.sender {
            Some(sender) => match sender.send(1) {
                Ok(_) => {}
                Err(error) => {
                    return Err(Error::String(format!(
                        "Failed to cancel serial port data reading: {}",
                        error
                    )));
                }
            },
            None => {}
        }
        serialport_info.sender = None;
        println!("Cancel {} serial port reading", &path);
        Ok(())
    })
}

/// `close` closes the specified serial port
#[tauri::command]
pub fn close<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    state: State<'_, SerialportState>,
    path: String,
) -> Result<(), Error> {
    match state.serialports.lock() {
        Ok(mut serialports) => {
            if serialports.remove(&path).is_some() {
                Ok(())
            } else {
                Err(Error::String(format!("Serial port {} is not open!", &path)))
            }
        }
        Err(error) => Err(Error::String(format!("Failed to acquire lock: {}", error))),
    }
}

/// `close_all` close all serial ports
#[tauri::command]
pub fn close_all<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    state: State<'_, SerialportState>,
) -> Result<(), Error> {
    match state.serialports.lock() {
        Ok(mut map) => {
            for serialport_info in map.values() {
                if let Some(sender) = &serialport_info.sender {
                    match sender.send(1) {
                        Ok(_) => {}
                        Err(error) => {
                            println!("Failed to cancel serial port data reading: {}", error);
                            return Err(Error::String(format!(
                                "Failed to cancel serial port data reading: {}",
                                error
                            )));
                        }
                    }
                }
            }
            print!("Serial ports closed");
            map.clear();
            Ok(())
        }
        Err(error) => Err(Error::String(format!("Failed to acquire lock: {}", error))),
    }
}

/// `force_close` forcibly close the serial port
#[tauri::command]
pub fn force_close<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    state: State<'_, SerialportState>,
    path: String,
) -> Result<(), Error> {
    match state.serialports.lock() {
        Ok(mut map) => {
            if let Some(serial) = map.get_mut(&path) {
                if let Some(sender) = &serial.sender {
                    match sender.send(1) {
                        Ok(_) => {}
                        Err(error) => {
                            println!("Failed to cancel serial port data reading: {}", error);
                            return Err(Error::String(format!(
                                "Failed to cancel serial port data reading: {}",
                                error
                            )));
                        }
                    }
                }
                map.remove(&path);
                Ok(())
            } else {
                Ok(())
            }
        }
        Err(error) => Err(Error::String(format!("Failed to acquire lock: {}", error))),
    }
}

/// `open` opens the specified serial port
#[tauri::command]
pub fn open<R: Runtime>(
    _app: AppHandle<R>,
    state: State<'_, SerialportState>,
    _window: Window<R>,
    path: String,
    baud_rate: u32,
    data_bits: Option<usize>,
    flow_control: Option<String>,
    parity: Option<String>,
    stop_bits: Option<usize>,
    timeout: Option<u64>,
) -> Result<(), Error> {
    println!("open: {:}", path);
    match state.serialports.lock() {
        Ok(mut serialports) => {
            if serialports.contains_key(&path) {
                return Err(Error::String(format!("Serial port {} is open!", path)));
            }
            match serialport::new(path.clone(), baud_rate)
                .data_bits(get_data_bits(data_bits))
                .flow_control(get_flow_control(flow_control))
                .parity(get_parity(parity))
                .stop_bits(get_stop_bits(stop_bits))
                .timeout(Duration::from_millis(timeout.unwrap_or(200)))
                .open()
            {
                Ok(serial) => {
                    let data = SerialportInfo {
                        serialport: serial,
                        sender: None,
                    };
                    serialports.insert(path, data);
                    Ok(())
                }
                Err(error) => Err(Error::String(format!(
                    "Failed to create {} serial port: {}",
                    path, error.description
                ))),
            }
        }
        Err(error) => Err(Error::String(format!("Failed to acquire lock: {}", error))),
    }
}

/// `read` read the specified serial port
#[tauri::command]
pub fn read<R: Runtime>(
    _app: AppHandle<R>,
    window: Window<R>,
    state: State<'_, SerialportState>,
    path: String,
    timeout: Option<u64>,
    size: Option<usize>,
) -> Result<(), Error> {
    let event_path = path.replace(".", "");
    let disconnected_event = format!("plugin-serialport-disconnected-{}", &event_path);
    get_serialport(state.clone(), path.clone(), |serialport_info| {
        if serialport_info.sender.is_some() {
            println!("Serial port {} is already reading data!", &path);
            Ok(())
        } else {
            println!("Serial port {} starts reading data!", &path);
            match serialport_info.serialport.try_clone() {
                Ok(mut serial) => {
                    let event_path = path.replace(".", "");
                    let read_event = format!("plugin-serialport-read-{}", &event_path);
                    println!("event: {}", &read_event);
                    let (tx, rx): (Sender<usize>, Receiver<usize>) = mpsc::channel();
                    serialport_info.sender = Some(tx);
                    thread::spawn(move || {
                        let mut message_buf = String::new(); // Buffer to store the message
                        loop {
                            // Check if a signal has been received to stop reading
                            match rx.try_recv() {
                                Ok(_) | Err(TryRecvError::Disconnected) => {
                                    // If a signal is received or the channel is disconnected, break the loop and exit
                                    println!("Received stop signal for serial port {}", &path);
                                    break;
                                }
                                _ => {} // Continue reading data if no signal received
                            }
                            let mut buf = [0; 1]; // Buffer to read a single byte
                            match serial.read_exact(&mut buf) {
                                Ok(_) => {
                                    // Convert the byte to a character
                                    let character = buf[0] as char;
                                    // Append the character to the message buffer
                                    message_buf.push(character);
                                    
                                    // Check if a newline character is encountered, indicating the end of a message
                                    if character == '\n' {
                                        // Emit the complete message to the frontend
                                        match window.emit(&read_event, ReadData {
                                            data: message_buf.as_bytes(),
                                            size: message_buf.len(),
                                        }) {
                                            Ok(_) => {}
                                            Err(error) => {
                                                println!("Failed to send data: {}", error)
                                            }
                                        }
                                        
                                        // Clear the message buffer to prepare for the next message
                                        message_buf.clear();
                                    }
                                }
                                Err(ref err) if err.kind() == ErrorKind::TimedOut => {
                                    // Timed out, continue waiting for data
                                    continue;
                                }
                                Err(err) => {
                                    println!("Failed to read from serial port: {:?}", err);
                                    break; // Break out of the loop for other errors
                                }
                            }
                        }
                    });
                }
                Err(error) => {
                    match window.emit(
                        &disconnected_event,
                        format!("Serial port {} disconnected!", &path),
                    ) {
                        Ok(_) => {}
                        Err(error) => {
                            println!("Failed to send disconnection event: {}", error)
                        }
                    }
                    return Err(Error::String(format!(
                        "Failed to read {} serial port: {}",
                        &path, error
                    )));
                }
            }
            Ok(())
        }
    })
}

/// `write` writes to the specified serial port
#[tauri::command]
pub fn write<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    state: State<'_, SerialportState>,
    path: String,
    value: String,
) -> Result<usize, Error> {
    let event_path = path.replace(".", "");
    let disconnected_event = format!("plugin-serialport-disconnected-{}", &event_path);
    // Print the string that will be written to the serial port
    println!("Writing to serial port {}: {}", path, value);
    get_serialport(state, path.clone(), |serialport_info| match serialport_info
        .serialport
        .write(value.as_bytes())
    {
        Ok(size) => Ok(size),
        Err(error) => {
            match _window.emit(
                &disconnected_event,
                format!("Serial port {} disconnected!", &path),
            ) {
                Ok(_) => {}
                Err(error) => {
                    println!("Failed to send disconnection event: {}", error)
                }
            }
            Err(Error::String(format!(
                "Failed to write data to serial port {}: {}",
                &path, error
            )))
        }
    })
}

/// `write` write binary content to the specified serial port
#[tauri::command]
pub fn write_binary<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    state: State<'_, SerialportState>,
    path: String,
    value: Vec<u8>,
) -> Result<usize, Error> {
    get_serialport(state, path.clone(), |serialport_info| match serialport_info
        .serialport
        .write(&value)
    {
        Ok(size) => Ok(size),
        Err(error) => Err(Error::String(format!(
            "Failed to write data to serial port {}: {}",
            &path, error
        ))),
    })
}
