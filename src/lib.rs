// #![deny(warnings)]
#![allow(non_snake_case)]

use std::env;
use std::collections::HashMap;

use std::error;
use std::fmt;

use std::default::Default;
use std::convert::Infallible;

use std::cmp::Ordering;

use rand::distributions::{Distribution, Uniform};

use Monies::{Money, define_currency_set};

define_currency_set!(
  internal {
    INV: {
      code: "INV",
      exponent: 2,
      locale: Locale::EnUs,
      minor_units: 100,
      name: "INV",
      symbol: "𝒾",
      symbol_first: true
    }
  }
);

/// Enum of built in Error types
#[derive(Debug,PartialEq,Clone, Copy,Serialize, Deserialize)]
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
        WindmillError::Incomplete
    }
}

impl Default for RanIDs {
    fn default() -> RanIDs {
        RanIDs::new()
    }
}

#[derive(Debug,PartialEq,Clone, Serialize, Deserialize)]
struct Asset {
    name: String,
    address: String,
}

#[derive(Debug,Eq,Clone, Serialize, Deserialize)]
struct Bid {
    id: String,
    quantity: u64,
    price: Money,
    timestamp: DateTime<Utc>,
}

impl Ord for Bid {
    fn cmp(&self, other: &Self) -> Ordering {
        let ord = (self.price/self.quantity).cmp((&other.price/&other.quantity));
        match ord {
            Ordering::Equal => return &other.timestamp.cmp(self.timestamp);
            _ => return ord;
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
        (self.price/self.quantity) == (&other.price/&other.quantity)) &&
        &other.timestamp == self.timestamp
    }
}

#[derive(Debug,PartialEq,Clone, Serialize, Deserialize)]
enum AuctionResult {
    Success {quantity: u64, price: Money},
    InProgress,
    Failure,
}

#[derive(Debug,PartialEq,Clone, Serialize, Deserialize)]
enum BidResult {
    Completed(AuctionResult),
    Submitted,
}

#[derive(Debug,PartialEq,Clone,Serialize, Deserialize)]
struct Auction {
    shares: u64,
    tot: u64,
    bids: HashMap<string,Bid>,
    results: HashMap<string,AuctionResult>,
    ids: RanIDs,
}

impl Auction {
    fn new(shares: u64) -> Self {
        Auction {
            shares,
            tot: 0,
            bids: HashMap::default(),
            results: HashMap::default(),
            ids: RanIDs::default(),
        }
    }

    fn join(&mut self) -> Result<String,WindmillError> {
        let nt = self.ids.get()?;
        self.results.insert(nt.clone(),AuctionResult::InProgress);
        nt
    }

    fn bid(&mut self,bid: Bid) -> Result<BidResult,WindmillError> {
        let {id, quantity, price} = bid.clone();
        if let Some(res) = self.results.get(&id.clone()){
            return match *res {
                AuctionResult::InProgress => {
                    tot += quantity;
                    bids.insert(&id.clone(),bid);
                    if tot < shares {
                        return Ok(BidResult::Submitted);
                    } else {
                    //TODO
                    }
                }
                _ => return Ok(BidResult::Completed(*res));
        }
        else {
            return Err(WindmillError::BadRequest);
        }
    }

    fn tabulate(&mut self) -> AuctionResult {
        let mut final_results = self.bids.values().cloned().collect();
        final_results.sort_unstable().reverse();
        let mut counted = 0u64;
        for bb in final_results.iter() {
            if counted + bb.quantity <= self.shares {
                if let Some(res) = self.results.get(&id.clone()){
                    match *res {
                        AuctionResult::InProgress => {
                            *res = AuctionResult::Success{quantity: *bb.quantity, price: Money::from_minor(0,internal::INV)};
                        }
                        AuctionResult::Success{qq, _} => {
                            *res = AuctionResult::Success{quantity: qq+*bb.quantity, price: Money::from_minor(0,internal::INV)};
                        }
                    }
                }
            }
        }
    }
}
