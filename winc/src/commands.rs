/*
MIT License

Copyright (c) 2022 OmiBYTE.io

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
 */

use crate::bsp::{Pin, Spibus};
use crate::crc7::crc7;

pub const CMD_DMA_WRITE: u8 = 0xC1;
pub const CMD_DMA_READ: u8 = 0xC2;
pub const CMD_REGISTER_INTERNAL_WRITE: u8 = 0xC3;
pub const CMD_REGISTER_INTERNAL_READ: u8 = 0xC4;
pub const CMD_TRANSACTION_TERMINATE: u8 = 0xC5;
pub const CMD_REPEAT_DATA_PACKET: u8 = 0xC6;
pub const CMD_DMA_EXTENDED_WRITE: u8 = 0xC7;
pub const CMD_DMA_EXTENDED_READ: u8 = 0xC8;
pub const CMD_SINGLE_WORD_WRITE: u8 = 0xC9;
pub const CMD_SINGLE_WORD_READ: u8 = 0xCA;
pub const CMD_SOFT_RESET: u8 = 0xCF;

pub const LEN_COMMAND_A: u8 = 4;
pub const LEN_COMMAND_B: u8 = 6;
pub const LEN_COMMAND_C: u8 = 7;
pub const LEN_COMMAND_D: u8 = 8;

enum CommandError {
    ErrNone = 0x00,
    ErrUnsupportedCommand = 0x01,
    ErrUnexpectedData = 0x2,
    ErrCmdCrc7Error = 0x3,
    ErrDataCrc7Error = 0x4,
    ErrInternalError = 0x05,

    // Specific to this driver implementation
    ErrCommandMismatch = 0xF0,
}

impl CommandError {
    fn from_u8(value: u8) -> CommandError {
        match value {
            0x00 => CommandError::ErrNone,
            0x01 => CommandError::ErrUnsupportedCommand,
            0x02 => CommandError::ErrUnexpectedData,
            0x03 => CommandError::ErrCmdCrc7Error,
            0x04 => CommandError::ErrDataCrc7Error,
            0x05 => CommandError::ErrInternalError,

            0xF0 => CommandError::ErrCommandMismatch,

            _ => panic!("Unknown value: {}", value)
        }
    }
}

//****************************
// Base trait for all commands
//****************************
trait Command<Transport, PinType>
where
    Transport: Spibus<PinType>,
    PinType: Pin,
{
    const LENGTH: usize;
    const TYPE_CODE: u8;

    fn write(mut spi: Transport, data: [u8; Self::LENGTH], crc: bool) {
        // Write the command payload
        spi.tx(&data[..]);

        // Write CRC7 if enabled
        if crc {
            // Calculate CRC7 of command payload
            let crc7 = crc7(&data[..]);

            // Send the CRC.
            // NOTE: Bit 0 should be 1 and 1- 7 should be the CRC7.
            spi.transfer((crc7 << 1) | 0x01);
        }
    }

    fn response(&self, mut spi: Transport) -> Result<bool, CommandError> {
        if Self::TYPE_CODE == CMD_SOFT_RESET
            || Self::TYPE_CODE == CMD_TRANSACTION_TERMINATE
            || Self::TYPE_CODE == CMD_REPEAT_DATA_PACKET
        {
            // Read (and ignore) leading byte
            spi.transfer(0);
        }

        // The response is always 2 bytes
        let mut resp: [u8; 2] = [0, 0];

        // Receive the response
        spi.rx(&mut resp[..]);

        // First byte must match that of the command type code
        if resp[0] != Self::TYPE_CODE {
            return Err(CommandError::ErrCommandMismatch);
        }

        // Half of the second byte is the error code
        let err: CommandError = CommandError::from_u8(resp[1] & 0x0F);

        // Return the error if one occurred or success
        return match err {
            CommandError::ErrNone => Ok(true),
            _ => Err(err)
        }
    }
}


