use std::error;
use std::convert::TryFrom;
use std::thread;

extern crate chrono;

use chrono::prelude::*;
use chrono::Duration;

use rusty_money::{Money};

use rand::distributions::{Distribution, Uniform};

use windmill::{Auction, Bid, BidResult, AuctionResult, chain};


fn main() -> Result<(),Box<dyn error::Error>> {
    let range = Uniform::from(100..1000);

    let mut auction = Auction::new(Utc::now() + Duration::seconds(10), 1_000u64);
    for _bidder in 0..10 {
        if let Ok(my_id) = auction.join() {
            let mut rng =  rand::thread_rng();
            let q: u64 = range.sample(&mut rng);
            let p: i64 = i64::try_from(range.sample(&mut rng)).unwrap();
            let bb = Bid { id: my_id, quantity: q, price: Money::from_major(p,chain::INV), timestamp: Utc::now()};
            // eprintln!("{:?}", bb);
            if let Ok(br) = auction.bid(bb){
                match br {
                    BidResult::Submitted => thread::sleep(Duration::seconds(2).to_std().unwrap()),
                    _ => {},
                }
                eprintln!("{:?}",br);

            }
        }
    }
    // auction.tabulate();
    eprintln!("{}", "TABULATED");
    for (key, val) in auction.results.iter_mut() {
        match val {
            AuctionResult::Success{quantity,price} => eprintln!("{}: {}, {}", key,quantity,price),
            AuctionResult::Failure => eprintln!("{}: {}", key,"Failure"),
            _ => {},
        }

    }
    Ok(())
}
