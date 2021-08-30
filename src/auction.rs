extern crate rusty_money;

use std::collections::HashMap;
use std::collections::hash_map::Iter;

use std::convert::TryInto;

use std::default::Default;

use chrono::prelude::*;

use rusty_money::Money;

use crate::types::{Bid, BidResult, AuctionResult, Chain, WindmillError};
use crate::RanIDs;

#[derive(Clone)]
pub struct Auction {
    time: DateTime<Utc>,
    shares: u64,
    bids: HashMap<String,Bid>,
    results: HashMap<String,AuctionResult>,
    ids: RanIDs,
    completed: bool,
}

impl Auction {
    pub fn new(time: DateTime<Utc>, shares: u64) -> Self {
        Auction {
            time,
            shares,
            bids: HashMap::default(),
            results: HashMap::default(),
            ids: RanIDs::default(),
            completed: false,
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

    pub fn check_bid(&self, key: &String) -> Option<&Bid> {
        self.bids.get(key)
    }

    pub fn check_result(&self, key: &String) -> Option<&AuctionResult> {
        self.results.get(key)
    }



    pub fn tabulate(&mut self) -> bool {
        if !self.completed {
            self.completed = true;
            let mut final_results: Vec<Bid> = self.bids.values().cloned().collect();
            final_results.sort_unstable();
            final_results.reverse();
            let mut price = Money::from_minor(0,Chain::INV);
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
        self.completed
    }
}

impl<'a> IntoIterator for &'a Auction {
    type Item = (&'a String, &'a AuctionResult);
    type IntoIter = Iter<'a, String, AuctionResult>;

    fn into_iter(self) -> Self::IntoIter {
        self.results.iter()
    }
}
