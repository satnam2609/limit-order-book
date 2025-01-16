use std::fmt::{self, Debug};
use std::ptr;
use std::sync::atomic::{AtomicPtr, AtomicU32, Ordering::*};
use chrono::{DateTime, Utc};
use serde::{Serialize,Deserialize};



/// Order type is used to refer the type of [`Order`]
#[derive(Debug, Clone, Copy, PartialEq, Eq,Serialize,Deserialize)]
pub enum OrderType {
    BID, // When the order is for buying the stock
    ASK, // WHen the order is for selling the stock
}

/// This enum is used to manage the state of [`Order`].
#[derive(Debug, Clone, Copy, PartialEq, Eq,Serialize,Deserialize)]
pub enum OrderStatus {
    WAIT,
    /// means the order is not executed but is present in the limit order book.
    PARTIAL, // means the order is only partially executed.
    FULL, // means the order is fully executed and not present in the limit order book.
    // but that order will be saved by the system for analytics.
    CANCEL, // means the order was cancelled and is not present in the limit order book.
            // but the user can view the order from their order history.
}



/// This struct will holds all the data about the order and
/// and his used as a Atomic Linked list for travesing in the list
/// by the Limit node. 
#[derive(Serialize,Deserialize)]
pub struct Order {
    pub seqeunce: i32,         // Unique number for identification
    pub order_type: OrderType, // Order type enum
    pub price: f64,            // Limit price of the Order
    pub shares: AtomicU32,     // This field is Atomic because on concurrent
    // operations the current shares might change
    #[serde(skip_deserializing)]
    pub entry_time: DateTime<Utc>, // Entry time of the order
    #[serde(skip_serializing,skip_deserializing)]
    pub order_status: AtomicPtr<OrderStatus>, // Order status enum
    #[serde(skip_serializing,skip_deserializing)]
    pub next:AtomicPtr<Order>  ,    
    #[serde(skip_serializing,skip_deserializing)]                          // Atomic pointer to the next node in the list
    pub prev:AtomicPtr<Order> ,
                            
}



impl Clone for Order {
    fn clone(&self) -> Order {
        Order {
            seqeunce: self.seqeunce,
            order_type: self.order_type,
            price: self.price,
            shares: AtomicU32::new(self.shares.load(SeqCst)),
            entry_time: self.entry_time,
            order_status: AtomicPtr::new(self.order_status.load(SeqCst)),
            next:AtomicPtr::new(self.next.load(Acquire)),
            prev:AtomicPtr::new(self.prev.load(Acquire)),
        }
    }
}

impl Debug for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Order:seq;{:?} type:{:?}, price:{:?},shares:{:?}, entry_time:{:?} , status:{:?}",
            self.seqeunce,
            self.order_type,
            self.price,
            self.shares,
            self.entry_time,
            self.order_status,
        )
    }
}

impl Order {
    pub fn new(seq: i32, order_type: OrderType, price: f64, shares: u32) -> Order {
        Order {
            seqeunce: seq,
            order_type,
            price,
            shares: AtomicU32::new(shares),
            order_status: AtomicPtr::new(&mut OrderStatus::WAIT),
            entry_time: Utc::now(),
            next:AtomicPtr::new(ptr::null_mut()),
            prev:AtomicPtr::new(ptr::null_mut()),
        }
    }
}

#[cfg(test)]
mod tests {}
