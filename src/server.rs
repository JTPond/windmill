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
use warp::{Filter, Server};
use serde_derive::{Deserialize, Serialize};

use chrono::prelude::*;

use rust_decimal::prelude::*;

use windmill::Auction;
use windmill::RanIDs;
use windmill::types::{Bid, BidResult, AuctionResult, WindmillError};

type Seller = Arc<RwLock<Option<mpsc::UnboundedSender<Result<Message, warp::Error>>>>>;
type Bidders = Arc<RwLock<HashMap<String, Option<mpsc::UnboundedSender<Result<Message, warp::Error>>>>>>;

#[derive(Default)]
struct Room {
    pub asset: String,
    pub auction: Auction,
    pub seller: Seller,
    pub bidders: Bidders,
}

type Rooms = Arc<RwLock<HashMap<String, Room>>>;

#[derive(Debug,PartialEq,Clone, Serialize, Deserialize)]
enum RequestMessage {
    NewRoom { asset: String, shares: u64, end_time: DateTime<Utc>, seller_token: String},
}

#[derive(Debug,PartialEq,Clone, Serialize, Deserialize)]
enum ResponseMessage {
    NewRoom(String),
    NewUser(String),
    RoomDetails { asset: String, shares: u64, end_time: DateTime<Utc> },
    Error(WindmillError),
}


type IDs = Arc<RwLock<RanIDs>>;

pub fn start() -> Server {
    let mut token: String = String::from("");
    let mut tids = RanIDs::new();
    for _i in 0..3 {
        if let Ok(nt) = tids.get() {
            token.push_str(&nt);
        }
    }
    if let Ok(val) = env::var("WINDMILL_TEST") {
        if val == "1" {
            token = String::from("000000000000000");
        }
    }

    eprintln!("Host Token: {}",token);

    let token = warp::any().map(move || token.clone());

    let ids: IDs = IDs::default();
    let ids = warp::any().map(move || -> IDs {
        return ids.clone();
    });

    let rooms = Rooms::default();
    let rooms = warp::any().map(move || rooms.clone());

    let new_room = warp::path("new")
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .and(rooms.clone())
        .and(ids.clone())
        .and(token.clone())
        .and_then(new_room);

    let check = warp::path("check")
        .and(warp::path::param())
        .and(rooms.clone())
        .and_then(check_room);

    let join = warp::path("join")
        .and(warp::path::param())
        .and(rooms.clone())
        .and(ids.clone())
        .and_then(new_user);

    // GET /chat -> websocket upgrade
    let chat = warp::path("chat")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::path::param())
        .and(warp::path::param())
        .and(rooms.clone())
        .and(client.clone())
        .and(warp::ws())
        .map(|rid: String, uid: String, rooms: Rooms, client: AClient, ws: warp::ws::Ws| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| user_connected(socket, rid, uid, rooms, client))
        });

    let watch = warp::path("watch")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::path::param())
        .and(warp::path::param())
        .and(token.clone())
        .and(rooms.clone())
        .and(warp::ws())
        .map(|rid: String, host_token: String, token: String, rooms: Rooms, ws: warp::ws::Ws| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| host_connected(socket, rid, rooms, host_token, token))
        });

    let routes = (warp::post().and(new_room))
              .or(warp::get().and(chat.or(watch).or(check).or(join).or(host_index).or(host_js).or(host_css)
                                               .or(guest_index).or(guest_js).or(guest_css)));

    warp::serve(routes)

}
