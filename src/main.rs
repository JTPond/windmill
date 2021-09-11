use std::env;
use std::collections::HashMap;
use std::sync::Arc;

use std::error;
use std::fmt;

use std::default::Default;
use std::convert::Infallible;

use rand::distributions::{Distribution, Uniform};

use futures::{FutureExt, StreamExt};
use tokio::sync::{mpsc, RwLock};
use warp::ws::{Message, WebSocket};
// use warp::http::StatusCode;
use warp::Filter;
use serde_derive::{Deserialize, Serialize};

use chrono::prelude::*;

use windmill::Auction;
use windmill::types::{Bid, BidResult, AuctionResult};

fn main() -> Result<(),Box<dyn error::Error>> {

    Ok(())
}
