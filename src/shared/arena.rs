use std::collections::HashMap;

use crate::{borrow_var, pxs_debug, shared::{PtrMagic, var::{pxs_Var, pxs_VarT}}};

/// Wrapper around pxs_VarT for Arena
struct PAVar {
    ptr: pxs_VarT
}

unsafe impl Send for PAVar {}
unsafe impl Sync for PAVar {}

/// A memory arena for `pxs_Var`s.
pub struct PixelArena {
    vars: HashMap<u32, PAVar>,
    next_id: u32,
    idx: u32
}

impl PixelArena {
    pub fn new(idx: u32) -> PixelArena {
        PixelArena { vars: HashMap::new(), next_id: 0, idx }
    }

    /// Add a new `pxs_Var` for arena tracking
    pub fn alloc(&mut self, var: pxs_VarT) {
        let bvar = borrow_var!(var);
        let idx = self.next_id;
        self.next_id += 1;
        bvar.idx = idx as i32;
        bvar.arena = self.idx as i32;
        self.vars.insert(idx, PAVar { ptr: var });
    }

    /// Remove a specific `pxs_Var` from the arena. i.e. already freed elsewhere.
    pub fn remove_var(&mut self, idx: u32) {
        let _ = self.vars.remove(&idx);
    }
}

impl PtrMagic for PixelArena {}

impl Drop for PixelArena {
    fn drop(&mut self) {
        #[cfg(feature = "pxs-debug")]
        let count = self.vars.len();
        pxs_debug!("Dropping {count} number of vars");

        for (_, v) in self.vars.drain() {
            if v.ptr.is_null() {
                continue;
            }

            let mut var = pxs_Var::from_raw(v.ptr);
            var.idx = -1;
            // Drops here
        }
    }
}
