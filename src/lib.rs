// #![deny(warnings)]
#![crate_name = "windmill"]
#![allow(non_snake_case)]

extern crate rusty_money;

use std::collections::HashMap;
use std::convert::TryInto;

use std::error;
use std::fmt;

use std::default::Default;

use std::cmp::Ordering;

use chrono::prelude::*;

use rand::distributions::{Distribution, Uniform};

// use serde_derive::{Deserialize, Serialize};

use rusty_money::{Money, define_currency_set};

define_currency_set!(
  chain {
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

#[derive(Clone)]
struct RanIDs {
    radix: u32,
    pub ids: Vec<u32>,
    pub dist: Uniform<u32>,
}

impl RanIDs {
    fn new() -> Self {
        RanIDs {
            radix: 36,
            ids: Vec::new(),
            dist: Uniform::from(1..36_u32.pow(5)-1),
        }
    }

    fn get(&mut self) -> Result<String,WindmillError> {
        let mut rng =  rand::thread_rng();
        let mut x: u32 = self.dist.sample(&mut rng);
        while self.ids.contains(&x) {
            x = self.dist.sample(&mut rng);
        }
        self.ids.push(x);
        let mut result: [char;5] = [std::char::from_digit(0,self.radix).unwrap(); 5];

        let mut i = 5;
        while i > 0 {
            let m = x % self.radix;
            x = x / self.radix;

            result[i-1] = std::char::from_digit(m, self.radix).unwrap();
            if x == 0 {
                return Ok(result.iter().collect());
            }
            i -= 1;
        }
        Err(WindmillError::Incomplete)
    }
}

impl Default for RanIDs {
    fn default() -> RanIDs {
        RanIDs::new()
    }
}

#[derive(Debug,PartialEq,Clone)]
struct Asset {
    name: String,
    address: String,
}

#[derive(Debug,Eq,Clone)]
pub struct Bid {
    pub id: String,
    pub quantity: u64,
    pub price: Money<'static, chain::Currency>,
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
    Success {quantity: u64, price: Money<'static, chain::Currency>},
    InProgress,
    Failure,
}

#[derive(Debug,PartialEq,Clone,Copy)]
pub enum BidResult {
    PastTime,
    Submitted,
}

#[derive(Clone)]
pub struct Auction {
    time: DateTime<Utc>,
    shares: u64,
    bids: HashMap<String,Bid>,
    pub results: HashMap<String,AuctionResult>,
    ids: RanIDs,
}

impl Auction {
    pub fn new(time: DateTime<Utc>, shares: u64) -> Self {
        Auction {
            time,
            shares,
            bids: HashMap::default(),
            results: HashMap::default(),
            ids: RanIDs::default(),
        }
    }

    pub fn join(&mut self) -> Result<String,WindmillError> {
        let nt = self.ids.get()?;
        self.results.insert(nt.clone(),AuctionResult::InProgress);
        Ok(nt)
    }

    pub fn bid(&mut self,bid: Bid) -> Result<BidResult,WindmillError> {
        let Bid{id, quantity: _, price: _, timestamp} = bid.clone();
        if let Some(res) = self.results.get(&id.clone()){
            let local_res = res.clone();
            match local_res {
                AuctionResult::InProgress => {
                    if timestamp < self.time {
                        self.bids.insert(id.clone(),bid);
                        return Ok(BidResult::Submitted);
                    } else {
                        self.tabulate();
                        return Ok(BidResult::PastTime);
                    }
                }
                _ => return Ok(BidResult::PastTime),
            }
        }
        else {
            return Err(WindmillError::BadRequest);
        }
    }

    pub fn tabulate(&mut self) {
        let mut final_results: Vec<Bid> = self.bids.values().cloned().collect();
        final_results.sort_unstable();
        final_results.reverse();
        let mut price = Money::from_minor(0,chain::INV);
        let mut counter = 0usize;
        let mut counted = 0i64;
        let mut winners: HashMap<String,Bid> = HashMap::new();
        while counted < self.shares.try_into().unwrap() && counter < final_results.len() {
            let mut bb = final_results[counter].clone();
            price = bb.price.clone()/bb.quantity;
            counted += bb.quantity as i64;
            if counted > self.shares.try_into().unwrap() {
                bb.quantity = ((self.shares as i64) - counted + (bb.quantity as i64)) as u64;
            }
            winners.insert(bb.id.clone(),bb);
            counter += 1;
        }
        if counted >= self.shares.try_into().unwrap() {
            for (key, val) in self.results.iter_mut() {
                if let Some(bb) = winners.get(&key.clone()){
                    *val = AuctionResult::Success{quantity: bb.quantity, price: bb.quantity*price.clone()};
                } else {
                    *val = AuctionResult::Failure;
                }
            }
        } else {
            for (_, val) in self.results.iter_mut() {
                *val = AuctionResult::Failure;
            }
        }
    }
}
