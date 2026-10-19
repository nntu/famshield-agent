use crate::error::DnsError;

pub const BUF_SIZE: usize = 4096;

pub struct BytePacketBuffer {
    pub buf: [u8; BUF_SIZE],
    pub pos: usize,
}

impl Default for BytePacketBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl BytePacketBuffer {
    pub fn new() -> Self {
        BytePacketBuffer {
            buf: [0; BUF_SIZE],
            pos: 0,
        }
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        let mut buf = Self::new();
        let len = data.len().min(BUF_SIZE);
        buf.buf[..len].copy_from_slice(&data[..len]);
        buf.pos = 0;
        buf
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn filled(&self) -> &[u8] {
        &self.buf[..self.pos]
    }

    pub fn step(&mut self, steps: usize) -> Result<(), DnsError> {
        if self.pos + steps > BUF_SIZE {
            return Err(DnsError::EndOfBuffer);
        }
        self.pos += steps;
        Ok(())
    }

    pub fn seek(&mut self, pos: usize) -> Result<(), DnsError> {
        if pos > BUF_SIZE {
            return Err(DnsError::EndOfBuffer);
        }
        self.pos = pos;
        Ok(())
    }

    pub fn read(&mut self) -> Result<u8, DnsError> {
        if self.pos >= BUF_SIZE {
            return Err(DnsError::EndOfBuffer);
        }
        let res = self.buf[self.pos];
        self.pos += 1;
        Ok(res)
    }

    pub fn get(&self, pos: usize) -> Result<u8, DnsError> {
        if pos >= BUF_SIZE {
            return Err(DnsError::EndOfBuffer);
        }
        Ok(self.buf[pos])
    }

    pub fn read_u16(&mut self) -> Result<u16, DnsError> {
        let res = ((self.read()? as u16) << 8) | (self.read()? as u16);
        Ok(res)
    }

    pub fn read_u32(&mut self) -> Result<u32, DnsError> {
        let res = ((self.read()? as u32) << 24)
            | ((self.read()? as u32) << 16)
            | ((self.read()? as u32) << 8)
            | (self.read()? as u32);
        Ok(res)
    }

    pub fn write_u8(&mut self, val: u8) -> Result<(), DnsError> {
        if self.pos >= BUF_SIZE {
            return Err(DnsError::BufferOverflow);
        }
        self.buf[self.pos] = val;
        self.pos += 1;
        Ok(())
    }

    pub fn write_u16(&mut self, val: u16) -> Result<(), DnsError> {
        self.write_u8((val >> 8) as u8)?;
        self.write_u8((val & 0xFF) as u8)?;
        Ok(())
    }

    pub fn write_u32(&mut self, val: u32) -> Result<(), DnsError> {
        self.write_u8(((val >> 24) & 0xFF) as u8)?;
        self.write_u8(((val >> 16) & 0xFF) as u8)?;
        self.write_u8(((val >> 8) & 0xFF) as u8)?;
        self.write_u8((val & 0xFF) as u8)?;
        Ok(())
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), DnsError> {
        if self.pos + bytes.len() > BUF_SIZE {
            return Err(DnsError::BufferOverflow);
        }
        self.buf[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
        self.pos += bytes.len();
        Ok(())
    }

    pub fn read_qname(&mut self, out_name: &mut String) -> Result<(), DnsError> {
        let mut pos = self.pos;
        let mut jumped = false;
        let mut max_jumps = 10;
        let mut delim = "";

        loop {
            if max_jumps == 0 {
                return Err(DnsError::InvalidName);
            }

            let len = self.get(pos)?;
            if (len & 0xC0) == 0xC0 {
                if !jumped {
                    self.seek(pos + 2)?;
                }
                let b2 = self.get(pos + 1)? as u16;
                let offset = (((len as u16) ^ 0xC0) << 8) | b2;
                pos = offset as usize;
                jumped = true;
                max_jumps -= 1;
                continue;
            }

            pos += 1;
            if len == 0 {
                break;
            }

            out_name.push_str(delim);
            let str_buffer = if pos + (len as usize) <= BUF_SIZE {
                &self.buf[pos..pos + (len as usize)]
            } else {
                return Err(DnsError::EndOfBuffer);
            };
            out_name.push_str(&String::from_utf8_lossy(str_buffer).to_ascii_lowercase());
            delim = ".";
            pos += len as usize;
        }

        if !jumped {
            self.seek(pos)?;
        }

        Ok(())
    }

    pub fn write_qname(&mut self, qname: &str) -> Result<(), DnsError> {
        for label in qname.split('.') {
            let len = label.len();
            if len > 63 {
                return Err(DnsError::Format("Label exceeds 63 characters".into()));
            }
            self.write_u8(len as u8)?;
            self.write_bytes(label.as_bytes())?;
        }
        self.write_u8(0)?;
        Ok(())
    }
}
