use alloc::boxed::Box;
use core::mem::MaybeUninit;

use windows_sys::Wdk::System::SystemServices::{
    KeAcquireInStackQueuedSpinLock, KeInitializeSpinLock, KeReleaseInStackQueuedSpinLock,
    KLOCK_QUEUE_HANDLE,
};

/// RAII guard wrapping KLOCK_QUEUE_HANDLE in a stable heap allocation from non-paged pool.
pub struct KLockQueueHandle {
    handle: Box<KLOCK_QUEUE_HANDLE>,
}

unsafe impl Send for KLockQueueHandle {}
unsafe impl Sync for KLockQueueHandle {}

/// Wrapper around Windows kernel spin lock (KSPIN_LOCK).
/// Allocates the lock in non-paged pool to maintain a stable address in kernel memory.
pub struct KSpinLock {
    lock: Box<usize>,
}

unsafe impl Send for KSpinLock {}
unsafe impl Sync for KSpinLock {}

impl KSpinLock {
    pub fn create() -> Self {
        let mut lock = Box::new(0usize);
        unsafe {
            KeInitializeSpinLock(lock.as_mut());
        }
        KSpinLock { lock }
    }

    /// Acquires the queued spinlock.
    /// The handle is allocated in non-paged pool via Box to guarantee
    /// a stable address across moves while holding the lock at DISPATCH_LEVEL.
    pub fn lock(&mut self) -> KLockQueueHandle {
        let mut handle =
            Box::new(unsafe { MaybeUninit::<KLOCK_QUEUE_HANDLE>::zeroed().assume_init() });
        unsafe {
            KeAcquireInStackQueuedSpinLock(self.lock.as_mut(), handle.as_mut());
        }
        KLockQueueHandle { handle }
    }

    /// Acquires the queued spinlock on the stack for zero-allocation critical sections.
    pub fn with_lock<R, F: FnOnce() -> R>(&mut self, f: F) -> R {
        unsafe {
            let mut handle: KLOCK_QUEUE_HANDLE = MaybeUninit::zeroed().assume_init();
            KeAcquireInStackQueuedSpinLock(self.lock.as_mut(), &mut handle);
            let res = f();
            KeReleaseInStackQueuedSpinLock(&mut handle);
            res
        }
    }
}

impl Drop for KLockQueueHandle {
    fn drop(&mut self) {
        unsafe {
            KeReleaseInStackQueuedSpinLock(self.handle.as_mut());
        }
    }
}
