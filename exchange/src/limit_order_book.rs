use std::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
};

use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

use crate::{
    rpc::exchange::{
        exchange_server, CancelOrderReply, CancelOrderRequest, SubmitOrderReply, SubmitOrderRequest,
    },
    types::{Id, Price, Symbol, Timestamp, Tokens},
    BuySell,
};

/// https://gist.github.com/halfelf/db1ae032dc34278968f8bf31ee999a25
struct LimitOrder {
    order_id: Id,
    shares: usize,
    entry_time: Timestamp,
    event_time: Option<Timestamp>,
}
impl LimitOrder {
    fn new(shares: usize) -> Self {
        Self {
            order_id: todo!(),
            shares,
            entry_time: todo!(),
            event_time: todo!(),
        }
    }
}

/// https://gist.github.com/halfelf/db1ae032dc34278968f8bf31ee999a25
struct Limit {
    price: Tokens,       // unnecessary?
    size: usize,         // difference between total_volume field? // unfilled order shares
    total_volume: usize, // actually filled orders
    symbol: Symbol,
    orders: VecDeque<LimitOrder>,
}
impl Limit {
    fn new(price: Tokens, symbol: Symbol, size: usize) -> Self {
        Limit {
            price,
            size,
            total_volume: 0,
            symbol,
            orders: [LimitOrder::new(size)].into(),
        }
    }

    fn insert_order(&mut self, size: usize) {
        self.orders.push_back(LimitOrder::new(size));
    }
}

/// https://gist.github.com/halfelf/db1ae032dc34278968f8bf31ee999a25
/// Source suggests exploring using a sparse array instead of a treemap
pub(crate) struct LimitBook<'a> {
    buy_tree: RwLock<BTreeMap<Tokens, Limit>>,
    sell_tree: RwLock<BTreeMap<Tokens, Limit>>,
    lowest_sell: Option<&'a Limit>,
    highest_buy: Option<&'a Limit>, // pointer to?

                                    // TODO:
                                    // past_orders:
}

impl<'a> LimitBook<'a> {
    pub(crate) fn new() -> Self {
        Self {
            buy_tree: Default::default(),
            sell_tree: Default::default(),
            lowest_sell: None,
            highest_buy: None,
        }
    }

    pub(crate) async fn place_order(
        &mut self,
        limit: Tokens,
        shares: usize,
        symbol: Symbol,
        action: BuySell,
    ) {
        let mut tree = match action {
            BuySell::Buy => self.buy_tree.write(),
            BuySell::Sell => self.sell_tree.write(),
        }
        .await;

        if tree.contains_key(&limit) {
            tree.get_mut(&limit).unwrap().insert_order(shares);
        } else {
            tree.insert(limit, Limit::new(limit, symbol, shares));
        }
    }
    pub(crate) fn execute_order() {
        // pop order from tree
        // move orders around account storage
        // update limit volume + size
        // need a way to query all volumes + sizes?  TODO for later
    }
    pub(crate) fn cancel_order() {}
}

#[tonic::async_trait]
impl exchange_server::Exchange for LimitBook<'static> {
    async fn submit_order(
        &self,
        req: Request<SubmitOrderRequest>,
    ) -> Result<Response<SubmitOrderReply>, Status> {
        Ok(Response::new(SubmitOrderReply {
            a: "LOL".to_string(),
        }))
    }

    async fn cancel_order(
        &self,
        req: Request<CancelOrderRequest>,
    ) -> Result<Response<CancelOrderReply>, Status> {
        Ok(Response::new(CancelOrderReply {
            a: "LOL".to_string(),
        }))
    }
}
