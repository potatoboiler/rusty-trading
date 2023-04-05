use std::{
    cmp::min,
    collections::{btree_map, BTreeMap, VecDeque},
    ops::Bound::{Excluded, Included, Unbounded},
    sync::Arc,
};

use tokio::sync::RwLock;
use tonic::{metadata::IterMut, Request, Response, Status};

use crate::{
    rpc::exchange::{
        exchange_server, CancelOrderReply, CancelOrderRequest, SubmitOrderReply, SubmitOrderRequest,
    },
    types::{Id, Price, Symbol, Timestamp, Tokens},
    BuySell,
};

/// https://gist.github.com/halfelf/db1ae032dc34278968f8bf31ee999a25
pub(crate) struct LimitOrder {
    order_id: Id,
    shares: usize,
    entry_time: Timestamp,
    event_time: Option<Timestamp>,
    live: bool,
}
impl LimitOrder {
    fn new(shares: usize) -> Self {
        Self {
            order_id: todo!(),
            shares,
            entry_time: todo!(),
            event_time: todo!(),
            live: true,
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
            lowest_sell: None, // ask price
            highest_buy: None, // bid price
        }
    }

    pub(crate) async fn execute_placed_limit_order(
        &mut self,
        limit: Tokens,
        shares: usize,
        symbol: Symbol,
        action: BuySell,
    ) {
        // TODO: be able to send to specific users based on... GUID?
        // TODO:

        // matching references:
        // http://web.archive.org/web/20120626161034/http://www.cmegroup.com/confluence/display/EPICSANDBOX/Match+Algorithms
        // https://stackoverflow.com/questions/13112062/which-are-the-order-matching-algorithms-most-commonly-used-by-electronic-financi
        // https://stackoverflow.com/questions/15603315/efficient-data-structures-for-data-matching
        // https://corporatefinanceinstitute.com/resources/capital-markets/matching-orders/
        let mut execution_tree = match action {
            BuySell::Sell => &self.buy_tree,
            BuySell::Buy => &self.sell_tree,
        }
        .write()
        .await;
        let query_limit = match action {
            // FIXME: buy tree contains buy orders, so when querying from sell limit, (+limit, +infinity)
            // sell tree econtains sell orders, so we want to see (-limit, +infinity)
            BuySell::Buy => limit,
            BuySell::Sell => -limit,
        };
        // TODO: fast path to place order if none exist? should not be needed as it is a one-time edge case?

        // FIXME: reverse the ordering on one of the execution trees so we can make this fully generic
        let execution_tree_iter = execution_tree.range_mut((Included(query_limit), Unbounded));

        let mut shares = shares;
        let mut transferred_tokens = 0_usize;
        let remainder_order: Option<LimitOrder> = 'matching: {
            for (&limit_price, limit) in execution_tree_iter {
                let order_iter = limit.orders.iter_mut();
                for order in order_iter {
                    if shares > 0 {
                        // TODO: verify that the buyer still has money (when copying over to buy side, make sure the buyer actually has money)
                        // TODO: implement collateral so that we don't need to do the above per round
                        let transferred_shares = min(order.shares, shares);

                        shares -= transferred_shares;
                        order.shares -= transferred_shares;
                        transferred_tokens += transferred_shares * (limit_price as usize);

                        // TODO: transfer money from buyer to seller
                        if order.shares == 0 {
                            // TODO: delete more elegantly, possibly using by refactoring order loop to retain_mut? Might have to use .all() instead to short-circuit.
                            order.live = false;
                        }
                    } else {
                        break 'matching None; // we will always reach this condition if the order is completely filled!
                    }
                }
                // TODO: construct new Limit Order and place this on the buy tree (need a message queue?)
                limit.orders.retain(|order| order.live); // TODO: terminate this upon seeing the first True value (because of in-order iteration)
                                                         // TODO: garbage collect limits as well, figure out tif this should be done in a separate procedure?
            }
            Some(LimitOrder::new(shares))
        };
        execution_tree.retain(|_, limit| limit.orders.len() > 0); // FIXME: see if in practice, this check takes up more time than just iterating over dead limits
        drop(execution_tree);

        // FIXME: beware deadlock? refactor by grabbing both locks on entrance
        if shares > 0 {
            match action {
                BuySell::Buy => &self.buy_tree,
                BuySell::Sell => &self.sell_tree,
            }
            .write()
            .await
            .entry(query_limit)
            .or_insert(Limit::new(limit, symbol, 0))
            .insert_order(shares);
        };
    }

    async fn insert_order(&mut self, limit_price: Tokens, order: LimitOrder, action: BuySell) {
        let mut opposite_tree = match action {
            BuySell::Buy => &self.buy_tree,
            BuySell::Sell => &self.sell_tree,
        }
        .write()
        .await;
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

// TODO: implement LMAX Disruptor architecture
