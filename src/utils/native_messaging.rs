// Logic copied from:
// https://github.com/neon64/chrome-native-messaging/blob/master/src/lib.rs
// (MIT Licensed)

// TODO: This could remove the need for the `events` module with traits

use serde::Serialize;
use std::fmt::Debug;

use crate::events::{MessageFromBrowser, MessageToError};
use std::io::{Read, Write};

use crate::log;

// Native messaging protocol:
//
// u32 length of the JSON message
// JSON message
pub fn read_message<R: Read>(mut input: R) -> Result<MessageFromBrowser, MessageToError> {
    let mut length_buffer = [0; 4];
    input
        .read_exact(&mut length_buffer)
        .map_err(|err| MessageToError::IoError {
            kind: err.kind().to_string(),
            message: format!("{}", err),
        })?;
    let length = u32::from_le_bytes(length_buffer);

    let mut message_buffer = vec![0; length as usize];
    input
        .read_exact(&mut message_buffer)
        .map_err(|err| MessageToError::IoError {
            kind: err.kind().to_string(),
            message: format!("{}", err),
        })?;

    serde_json::from_slice(&message_buffer).map_err(|err| MessageToError::JsonParseError {
        message: format!("{}", err),
    })
}

pub fn send_message<W: Write, S: Serialize + Debug>(
    mut output: W,
    message: &S,
) -> Result<(), &'static str> {
    // log(&format!("Sending message: {:?} ", &message));

    let message_buffer =
        serde_json::to_vec(message).map_err(|_| "Send: Failed to serialize message")?;
    let length = message_buffer.len() as u32;

    // log(&format!(
    //     "Message length: {} {} {}",
    //     length,
    //     length.to_le(),
    //     length.to_be()
    // ));

    output
        .write_all(&length.to_ne_bytes())
        .map_err(|_| "Send: Failed to write message length")?;
    output
        .flush()
        .map_err(|_| "Send: Failed to flush message")?;
    output
        .write_all(&message_buffer)
        .map_err(|_| "Send: Failed to write message")?;

    output
        .flush()
        .map_err(|_| "Send: Failed to flush message")?;

    Ok(())
}
