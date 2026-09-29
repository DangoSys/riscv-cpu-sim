use crate::{bus::Width, Access, Privilege};

const ADDRESS_MASK: u64 = (1 << 54) - 1;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Pmp {
    config: [u8; 16],
    address: [u64; 16],
}

impl Pmp {
    pub fn config(&self, index: usize) -> u8 {
        self.config[index]
    }

    pub fn address(&self, index: usize) -> u64 {
        self.address[index]
    }

    pub fn set_config(&mut self, index: usize, value: u8) {
        if self.config[index] & 0x80 != 0 {
            return;
        }
        let mut value = value & 0x9f;
        // R=0,W=1 is reserved; this implementation makes W read as zero.
        if value & 3 == 2 {
            value &= !2;
        }
        self.config[index] = value;
    }

    pub fn set_address(&mut self, index: usize, value: u64) {
        let locked = self.config[index] & 0x80 != 0;
        let locked_upper_tor = index < 15 && self.config[index + 1] & 0x98 == 0x88;
        if !locked && !locked_upper_tor {
            self.address[index] = value & ADDRESS_MASK;
        }
    }

    pub fn allows(&self, address: u64, width: Width, access: Access, mode: Privilege) -> bool {
        self.allows_range(address, width as usize, access, mode)
    }

    pub fn allows_range(&self, address: u64, bytes: usize, access: Access, mode: Privilege) -> bool {
        let end = address as u128 + bytes as u128;
        for index in 0..16 {
            let config = self.config[index];
            let encoded = self.address[index];
            let (start, limit) = match (config >> 3) & 3 {
                0 => continue,
                1 => {
                    let lower = if index == 0 { 0 } else { self.address[index - 1] };
                    ((lower as u128) << 2, (encoded as u128) << 2)
                }
                2 => ((encoded as u128) << 2, ((encoded as u128) << 2) + 4),
                3 => {
                    let size = 1u128 << (encoded.trailing_ones() + 3);
                    let start = ((encoded as u128) << 2) & !(size - 1);
                    (start, start + size)
                }
                _ => unreachable!(),
            };
            if start >= limit || address as u128 >= limit || end <= start {
                continue;
            }
            // The first overlapping entry must cover the entire access, even in M mode.
            if (address as u128) < start || end > limit {
                return false;
            }
            if mode == Privilege::Machine && config & 0x80 == 0 {
                return true;
            }
            let permission = match access {
                Access::Fetch => 4,
                Access::Load => 1,
                Access::Store => 2,
            };
            return config & permission != 0;
        }
        mode == Privilege::Machine
    }
}
