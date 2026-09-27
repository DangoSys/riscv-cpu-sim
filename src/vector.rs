use crate::{csr::VS, hart::Hart, Trap};

// Vector configuration is implemented independently of vector data instructions.
// The core does not advertise misa.V until the full vector instruction set exists.
#[derive(Clone, Debug)]
pub struct VectorConfig {
    pub(crate) vlen: u64,
    pub(crate) vl: u64,
    pub(crate) vlmax: u64,
    pub(crate) vtype: u64,
    pub(crate) vstart: u64,
    pub(crate) vcsr: u64,
}

impl VectorConfig {
    pub fn new(vlen_bits: usize) -> Self {
        assert!((128..=65536).contains(&vlen_bits) && vlen_bits.is_power_of_two());
        Self {
            vlen: vlen_bits as u64,
            vl: 0,
            vlmax: 0,
            vtype: 1 << 63,
            vstart: 0,
            vcsr: 0,
        }
    }
}

impl Hart {
    pub(crate) fn configure_vector(&mut self, instruction: u32, avl: u64, register_type: u64) -> Result<u64, Trap> {
        let illegal = Trap::illegal(instruction);
        if self.csrs.status & VS == 0 {
            return Err(illegal);
        }
        let config = self.csrs.vector.as_mut().ok_or(illegal)?;
        let rs1 = (instruction >> 15) & 31;
        let rd = (instruction >> 7) & 31;
        let immediate_avl = instruction >> 30 == 3;
        let vtype = if instruction >> 31 == 0 {
            u64::from(instruction >> 20)
        } else if immediate_avl {
            u64::from((instruction >> 20) & 0x3ff)
        } else if instruction >> 25 == 0x40 {
            register_type
        } else {
            return Err(illegal);
        };
        let sew = 8u64 << ((vtype >> 3) & 7);
        let lmul = vtype & 7;
        if vtype >> 8 != 0 || sew > 64 || lmul == 4 || (lmul >= 5 && sew > (64 >> (8 - lmul))) {
            config.vtype = 1 << 63;
            config.vl = 0;
            config.vlmax = 0;
        } else {
            let vlmax = if lmul <= 3 {
                (config.vlen / sew) << lmul
            } else {
                (config.vlen / sew) >> (8 - lmul)
            };
            if !immediate_avl && rs1 == 0 && rd == 0 && vlmax != config.vlmax {
                return Err(illegal);
            }
            let avl = if immediate_avl {
                rs1 as u64
            } else if rs1 != 0 {
                avl
            } else if rd != 0 {
                vlmax
            } else {
                config.vl
            };
            config.vtype = vtype;
            config.vlmax = vlmax;
            config.vl = avl.min(vlmax);
        }
        config.vstart = 0;
        self.csrs.status |= VS;
        Ok(config.vl)
    }
}
