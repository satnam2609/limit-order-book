use crate::order::*;
use std::fmt::{self, Debug};

use std::ptr;
use std::sync::atomic::{AtomicPtr, AtomicU32, Ordering::*};

/// This struct is the limit price node for the tree in the limit order book,
/// it holds the price and volume of shares,size currently present [`Order`] in the node.
/// It holds the pointers to the [`Order`] which is used for traversing or excessing the [`Order`].
pub struct Limit {
    pub price: f64,        //  limit price of the node
    pub volume: AtomicU32, // This field holds the atomic type because of
    // concurrent operations the volume might change
    pub size: AtomicU32,
    pub order_type: OrderType, // Order type enum
    pub head: AtomicPtr<Order>,// Atomic pointer to the head of the Doubly-linked-list.
    pub tail: AtomicPtr<Order>,// Atomic pointer to the tail of the Doubly-linked-list.
}

impl Debug for Limit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_shares = self.volume.load(Acquire);
        let size = self.size.load(Acquire);
        write!(
            f,
            "Limit:: price:{:?},volume:{:?},size:{:?}, order_type:{:?},len:",
            self.price, total_shares, size, self.order_type
        )
    }
}

impl Limit {
    /// This method creates new instance of [`Limit`].
    pub fn new(price: f64, order_type: OrderType) -> Limit {
        Limit {
            price,
            volume: AtomicU32::new(0),
            size: AtomicU32::new(0),
            order_type,
            head: AtomicPtr::new(ptr::null_mut()),
            tail: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// This method insertes new [`Order`] atomically.
    pub fn insert(&self, order: *mut Order) {
        self.size.fetch_add(1, SeqCst);
        loop {
            let current_tail = self.tail.load(Acquire);

            unsafe {
                (*order).prev.store(current_tail, Release);
                if !current_tail.is_null() {
                    (*current_tail).next.store(order, Release);
                }

                if self
                    .tail
                    .compare_exchange(current_tail, order, AcqRel, Relaxed)
                    .is_ok()
                {
                    if self.head.load(Acquire).is_null() {
                        self.head.store(order, Release);
                    }
                    break;
                }
            }
        }
    }

    pub fn pop(&self) {
        todo!()
    }

    /// This method removes [`Order`] atomically.
    pub fn remove(&self, order: *mut Order, status: &mut OrderStatus) {
        if let Some(o) = unsafe { order.as_ref() } {
            o.order_status.store(status, Release);
            if o.prev.load(Acquire).is_null() {
                // n is head
                self.head.store(o.next.load(Acquire), Release);
            } else {
                if let Some(prev) = unsafe { o.prev.load(Acquire).as_ref() } {
                    prev.next.store(o.next.load(Acquire), Release);
                }
            }

            if o.next.load(Acquire).is_null() {
                // n is tail
                self.tail.store(o.prev.load(Acquire), Release);
            } else {
                if let Some(next) = unsafe { o.next.load(Acquire).as_ref() } {
                    next.prev.store(o.prev.load(Acquire), Release);
                }
            }

            self.size.fetch_sub(1, SeqCst);
            self.volume.fetch_sub(o.shares.load(Acquire), SeqCst)   ;      
        }
    }
}
