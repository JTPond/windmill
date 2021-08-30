#![allow(non_snake_case)]

use std::error;
use std::fmt;

use std::cmp::Ordering;

use chrono::prelude::*;

use rusty_money::{Money, define_currency_set};

define_currency_set!(
  Chain {
    INV:{
      code: "INV",
      exponent: 2,
      locale: Locale::EnUs,
      minor_units: 100,
      name: "INV",
      symbol: "𝒾",
      symbol_first: false,
    }
  }
);

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

#[derive(Debug,Eq,Clone)]
pub struct Bid {
    pub id: String,
    pub quantity: u64,
    pub price: Money<'static, Chain::Currency>,
    pub timestamp: DateTime<Utc>,
}

impl Ord for Bid {
    fn cmp(&self, other: &Self) -> Ordering {
        let ord = (self.price.clone()/self.quantity).cmp(&(other.price.clone()/other.quantity));
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
        (self.price.clone()/self.quantity) == (other.price.clone()/other.quantity) &&
        other.timestamp == self.timestamp
    }
}

#[derive(Debug,PartialEq,Clone)]
pub enum AuctionResult {
    Success {quantity: u64, price: Money<'static, Chain::Currency>},
    InProgress,
    Failure,
}

#[derive(Debug,PartialEq,Clone,Copy)]
pub enum BidResult {
    PastTime,
    Submitted,
}
