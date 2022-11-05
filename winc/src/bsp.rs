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

use alloc::vec::Vec;

/// Traits for Board Support Package so that this driver can be used with most embedded frameworks

pub trait Spibus<PinType>
where PinType: Pin
{
    fn transfer(&self, input: u8) -> u8;
    fn cs_pin(&mut self) -> &mut PinType;

    fn tx(&mut self, buffer: &[u8]) {
        // Assert the chip select pin
        self.cs_pin().set_asserted(true);

        // Send byte by byte ignoring received bytes
        for b in buffer {
            self.transfer(*b);
        }

        // Dessert chip select pin
        self.cs_pin().set_asserted(false);
    }

    fn rx(&mut self, buffer: &mut [u8]) {
        // Assert the chip select Pin and dessert it after return
        self.cs_pin().set_asserted(true);

        // Transfer null char so that byte can be received
        for i in 0..buffer.len() {
            buffer[i] = self.transfer(0)
        }

        // Dessert chip select pin
        self.cs_pin().set_asserted(false);
    }

    fn transact(&mut self, buffer: &[u8]) -> Vec<u8> {
        // Create receive buffer that is the same size as the write buffer
        let mut recv: Vec<u8> = Vec::with_capacity(buffer.len());

        // Assert the chip select Pin and dessert it after return
        self.cs_pin().set_asserted(true);

        // Send and receive byte by byte
        for b in buffer {
            recv.push_within_capacity(self.transfer(*b))
                .expect("receive buffer is too small");
        }

        // Dessert chip select pin
        self.cs_pin().set_asserted(false);

        return recv;
    }
}

pub trait Pin {
    fn set_asserted(&mut self, on: bool);
}