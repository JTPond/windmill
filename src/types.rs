#![allow(non_snake_case)]

use std::error;
use std::fmt;

use std::cmp::Ordering;

use chrono::prelude::*;

use rust_decimal::prelude::*;

use serde_derive::{Deserialize, Serialize};

/// Enum of built in Error types
#[derive(Debug,PartialEq,Clone, Copy)]
pub enum WindmillError {
    Incomplete,
    BadRequest,
    ClosedAuction,

}

impl fmt::Display for WindmillError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WindmillError::Incomplete => write!(f,"User profile is incomplete."),
            WindmillError::BadRequest => write!(f,"Bad Request"),
            WindmillError::ClosedAuction => write!(f,"Closed Auction"),
        }

    }
}
impl error::Error for WindmillError {
    fn description(&self) -> &str {
        match *self {
            WindmillError::Incomplete => "Incomplete user profile.",
            WindmillError::BadRequest => "Bad Request.",
            WindmillError::ClosedAuction => "Auction is closed.",
        }
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

#[derive(Debug,Eq,Clone, Serialize, Deserialize)]
pub struct Bid {
    pub id: String,
    pub quantity: u64,
    pub price: Decimal,
    pub timestamp: DateTime<Utc>,
}

impl Ord for Bid {
    fn cmp(&self, other: &Self) -> Ordering {
        let ord = (self.price.clone()/Decimal::from(self.quantity)).cmp(&(other.price.clone()/Decimal::from(other.quantity)));
        match ord {
            Ordering::Equal => return other.timestamp.cmp(&self.timestamp),
            _ => return ord,
        }
    }
}

impl PartialOrd for Bid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Bid {
    fn eq(&self, other: &Self) -> bool {
        (self.price.clone()/Decimal::from(self.quantity)) == (other.price.clone()/Decimal::from(other.quantity)) &&
        other.timestamp == self.timestamp
    }
}


#[derive(Debug,PartialEq,Clone, Copy, Serialize, Deserialize)]
pub enum AuctionResult {
    Success {quantity: u64, price: Decimal},
    InProgress,
    Failure,
}

#[derive(Debug,PartialEq,Clone,Copy, Serialize, Deserialize)]
pub enum BidResult {
    PastTime,
    Submitted,
}
