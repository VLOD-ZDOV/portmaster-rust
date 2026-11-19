use alloc::vec::Vec;
use core::cell::UnsafeCell;
use wdk::rw_spin_lock::RwSpinLock;

pub struct ArrayHolder {
    data: UnsafeCell<Option<Vec<u8>>>,
    lock: RwSpinLock,
}

unsafe impl Send for ArrayHolder {}
unsafe impl Sync for ArrayHolder {}

impl ArrayHolder {
    pub const fn default() -> Self {
        Self {
            data: UnsafeCell::new(None),
            lock: RwSpinLock::default(),
        }
    }

    pub fn save(&self, data: &[u8]) {
        let _guard = self.lock.write_lock();
        unsafe {
            *self.data.get() = Some(data.to_vec());
        }
    }

    pub fn load(&self) -> Option<Vec<u8>> {
        let _guard = self.lock.write_lock();
        unsafe { (*self.data.get()).take() }
    }
}
