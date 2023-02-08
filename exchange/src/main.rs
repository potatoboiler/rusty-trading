use anyhow::Result;
use rpc::exchange::exchange_server;
use std::collections::HashMap;
use tonic::transport::Server;

use types::{Symbol, Tokens};

enum OrderType {
    Limit,
    Market,
}
// Order types?
// take profit
// stop loss, stop entry
// market order, limmit
// https://www.investopedia.com/trading-order-types-and-processes-4689649

enum Action {
    Buy,
    Sell,
}

mod types {
    pub(crate) type Tokens = u32; // 10000 Tokens ~ $1 USD
    pub(crate) type Timestaamp = u128;
    pub(crate) type Id = usize;
    pub(crate) type Symbol = String;
}

#[repr(transparent)]
struct Price {
    price: Tokens,
}

struct Account {
    owned: HashMap<Symbol, u32>,
}

struct MarketBook {}

mod limit_order_book {
    use std::{
        cell::Cell,
        collections::{BTreeMap, VecDeque},
    };

    use tonic::{Request, Response, Status};

    use crate::{
        rpc::exchange::{exchange_server, SubmitOrderReply, SubmitOrderRequest},
        types::{Id, Symbol, Tokens},
    };

    /// https://gist.github.com/halfelf/db1ae032dc34278968f8bf31ee999a25
    #[derive(Clone)]
    struct LimitOrder {
        order_id: Id,
        shares: usize,
        entry_time: usize,
        event_time: Option<usize>,
    }

    /// https://gist.github.com/halfelf/db1ae032dc34278968f8bf31ee999a25
    #[derive(Clone)]
    struct Limit {
        price: Tokens, // unnecessary?
        size: usize,   // difference between total_volume field?
        total_volume: usize,
        symbol: Symbol,
        orders: VecDeque<LimitOrder>,
    }
    impl Limit {
        fn new() -> Self {
            todo!()
        }
    }

    /// https://gist.github.com/halfelf/db1ae032dc34278968f8bf31ee999a25
    /// Source suggests exploring using a sparse array instead of a treemap
    pub(crate) struct LimitBook<'a> {
        buyTree: BTreeMap<Tokens, Limit>,
        sellTree: BTreeMap<Tokens, Limit>,
        lowestSell: Option<&'a Limit>,
        highestBuy: Option<&'a Limit>, // pointer to?
    }

    impl<'a> LimitBook<'a> {
        pub(crate) fn new() -> Self {
            todo!()
        }
    }

    #[tonic::async_trait]
    impl<'a: 'static> exchange_server::Exchange for LimitBook<'a> {
        async fn submit_order(
            &self,
            req: Request<SubmitOrderRequest>,
        ) -> Result<Response<SubmitOrderReply>, Status> {
            todo!()
        }
    }
}

pub mod rpc {
    pub mod exchange {
        include!("proto/exchange.rs");
    }
}

struct Order {
    id: usize,
    order_type: OrderType,
    shares: usize,
    limit: usize,
}

pub async fn start() -> Result<()> {
    let addr = "127.0.0.1:6969".parse().unwrap();
    let limit_book: limit_order_book::LimitBook = limit_order_book::LimitBook::new();

    tokio::spawn(async move { todo!() });

    let svc = exchange_server::ExchangeServer::new(limit_book);
    Server::builder().add_service(svc).serve(addr).await?;
    Ok(())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    println!("Hello, world!");
    Ok(())
}
