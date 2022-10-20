#[cfg(not(feature = "concurrent"))]
use locking::lock::NonSendLock;
#[cfg(not(feature = "concurrent"))]
pub type Locket<T> = NonSendLock<std::cell::RefCell<T>>;

#[cfg(feature = "concurrent")]
use locking::lock::SendLock;
#[cfg(feature = "concurrent")]
pub type Locket<T> = SendLock<parking_lot::RwLock<T>>;
