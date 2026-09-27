pub mod bus;
mod compressed;
pub use compressed::decode as expand_compressed;
pub mod csr;
mod float;
pub mod hart;
pub mod mmu;
pub mod pmp;
pub mod vector;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Privilege {
    User = 0,
    Supervisor = 1,
    Machine = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Access {
    Fetch,
    Load,
    Store,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Trap {
    pub cause: u64,
    pub value: u64,
}

impl Trap {
    pub fn illegal(instruction: u32) -> Self {
        Self {
            cause: 2,
            value: instruction.into(),
        }
    }
}

impl Access {
    pub fn misaligned(self, address: u64) -> Trap {
        Trap {
            cause: match self {
                Self::Fetch => 0,
                Self::Load => 4,
                Self::Store => 6,
            },
            value: address,
        }
    }

    pub fn fault(self, address: u64) -> Trap {
        Trap {
            cause: match self {
                Self::Fetch => 1,
                Self::Load => 5,
                Self::Store => 7,
            },
            value: address,
        }
    }

    pub fn page_fault(self, address: u64) -> Trap {
        Trap {
            cause: match self {
                Self::Fetch => 12,
                Self::Load => 13,
                Self::Store => 15,
            },
            value: address,
        }
    }
}
