use std::error;
use std::convert::TryFrom;

extern crate chrono;

use chrono::prelude::*;
use chrono::Duration;

use rusty_money::{Money, iso};

use rand::distributions::{Distribution, Uniform};

use windmill::{Auction, Bid, AuctionResult};


fn main() -> Result<(),Box<dyn error::Error>> {
    let mut tbids = 0;
    let range = Uniform::from(100..1000);

    let mut auction = Auction::new(Utc::now() + Duration::seconds(10), 1_000u64);
    for bidder in 0..10 {
        if let Ok(my_id) = auction.join() {
            let mut rng =  rand::thread_rng();
            let q: u64 = range.sample(&mut rng);
            let p: i64 = i64::try_from(range.sample(&mut rng)).unwrap();
            let bb = Bid { id: my_id, quantity: q, price: Money::from_major(p,iso::USD), timestamp: Utc::now()};
            eprintln!("{:?}", bb);

            auction.bid(bb);
        }
    }
    auction.tabulate();
    eprintln!("{:?}", "TABULATED");
    for (key, val) in auction.results.iter_mut() {
        match val {
            AuctionResult::Success{..} => eprintln!("{:?}: {:?}", key,val),
            _ =>{},
        }

    }
    Ok(())
}
