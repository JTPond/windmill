#![crate_name = "windmill"]

pub mod types;
mod ranid;
mod auction;
mod server;

pub use ranid::RanIDs;
pub use auction::Auction;

#[cfg(test)]
mod tests {
    use super::Auction;
    use super::types::{Bid, BidResult, AuctionResult};

    use std::error;
    use std::thread;

    extern crate chrono;

    use chrono::prelude::*;
    use chrono::Duration;

    use rand::distributions::{Distribution, Uniform};

    use rust_decimal::prelude::*;

    #[test]
    fn main() -> Result<(),Box<dyn error::Error>> {
        let range = Uniform::from(100..1000);

        let mut auction = Auction::new(Utc::now() + Duration::seconds(10), 1_000u64);
        for _bidder in 0..10 {
            if let Ok(my_id) = auction.join() {
                let mut rng =  rand::thread_rng();
                let q: u64 = range.sample(&mut rng);
                let p: Decimal = range.sample(&mut rng).into();
                let bb = Bid { id: my_id, quantity: q, price: p, timestamp: Utc::now()};
                // eprintln!("{:?}", bb);
                if let Ok(br) = auction.bid(bb){
                    match br {
                        BidResult::Submitted => thread::sleep(Duration::seconds(2).to_std().unwrap()),
                        _ => {auction.tabulate();},
                    }
                    eprintln!("{:?}",br);

                }
            }
        }
        // auction.tabulate();
        eprintln!("{}", "TABULATED");
        let mut failes = 0usize;
        let mut consist = true;
        let mut buf = Decimal::ZERO;
        for (key, val) in auction.into_iter() {
            match val {
                AuctionResult::Success{quantity,price} => {
                    let bid = auction.check_bid(key).unwrap();
                    assert!(quantity.clone() <= bid.quantity, "a = {}, b = {}", quantity.clone(), bid.quantity);
                    assert!(price.clone() <= bid.price, "a = {}, b = {}", price.clone(), bid.price);
                    if buf.is_zero() {
                        buf = price.clone()/Decimal::from(*quantity);
                    } else {
                        if buf != price.clone()/Decimal::from(*quantity) {
                            consist = false;
                        }
                    }
                },
                AuctionResult::Failure => {
                    eprintln!("{}: {}", key,"Failure");
                    failes += 1;
                },
                _ => {},
            }

        }
        assert!(consist, "Price was not consistant");
        assert!(failes < 10, "{} failes", failes);
        Ok(())
    }
}
