use etffi::ptr_magic::PtrMagic;

use crate::{shared::{var::{pxs_Var, pxs_VarT}}};

#[allow(non_camel_case_types)]
/// A memory arena for `pxs_Var`s.
pub struct pxs_PixelArena {
    vars: Vec<pxs_VarT>
}

impl pxs_PixelArena {
    pub fn new() -> pxs_PixelArena {
        pxs_PixelArena { vars: Vec::new() }
    }

    /// Add a new `pxs_Var` for arena tracking
    pub fn alloc(&mut self, var: pxs_VarT) {
        self.vars.push(var);
    }

    /// Remove a specific `pxs_Var` from the arena. i.e. already freed elsewhere.
    pub fn remove_var(&mut self, idx: u32) {
        if idx >= self.vars.len() as u32 {
            return;
        }
        self.vars.remove(idx as usize);
    }

    /// Get number of items currently in PixelArena
    pub fn num_of_args(&self) -> usize {
        self.vars.len()
    }
}

impl PtrMagic for pxs_PixelArena {}

impl Drop for pxs_PixelArena {
    fn drop(&mut self) {
        #[cfg(feature = "pxs-debug")] {
            let count = self.vars.len();
            crate::pxs_debug!("Dropping {count} number of vars");
        }
        for v in self.vars.drain(0..self.vars.len()) {
            let _ = pxs_Var::from_raw(v);
        }
        self.vars.clear();
    }
}
