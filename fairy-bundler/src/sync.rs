#[cfg(not(feature = "concurrent"))]
use locking::lock::NonSendLock;
#[cfg(not(feature = "concurrent"))]
pub type Lock<T> = NonSendLock<std::cell::RefCell<T>>;

#[cfg(feature = "concurrent")]
use locking::lock::SendLock;
#[cfg(feature = "concurrent")]
pub type Lock<T> = SendLock<parking_lot::RwLock<T>>;
