use std::{
    cmp::min,
    collections::{BTreeMap, HashMap, VecDeque},
    ops::Bound::{Included, Unbounded},
    sync::Arc,
};

use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{
    rpc::exchange::{
        exchange_server, CancelOrderReply, CancelOrderRequest, SubmitLimitOrderReply,
        SubmitLimitOrderRequest, SubmitMarketOrderReply, SubmitMarketOrderRequest,
    },
    types::{Id, Symbol, Timestamp, Tokens},
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
            order_id: Uuid::new_v4(),
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
    orders: VecDeque<LimitOrder>,
}
impl Limit {
    fn new(price: Tokens, symbol: Symbol, size: usize) -> Self {
        Limit {
            price,
            size,
            total_volume: 0,
            orders: [LimitOrder::new(size)].into(),
        }
    }

    fn insert_order(&mut self, size: usize) -> Uuid {
        let order = LimitOrder::new(size);
        let order_id = order.order_id.clone();
        self.orders.push_back(order);

        order_id
    }
}

type LimitBookTicker = Arc<RwLock<BTreeMap<Tokens, Limit>>>;
/// https://gist.github.com/halfelf/db1ae032dc34278968f8bf31ee999a25
/// Source suggests exploring using a sparse array instead of a treemap
pub(crate) struct LimitBook {
    buy_tickers: Arc<RwLock<HashMap<Symbol, LimitBookTicker>>>,
    sell_tickers: Arc<RwLock<HashMap<Symbol, LimitBookTicker>>>,
    // lowest_sell: Option<Arc<RefCell<Limit>>>,
    // highest_buy: Option<Arc<RefCell<Limit>>>, // pointer to?

    // TODO:
    // past_orders:
}

impl LimitBook {
    // TODO: update lowest_sell and highest_buy fields in order matching function
    // TODO: refactor to have different buy/sell trees for each symbol, but this should work for now (at low frequencies)

    pub(crate) fn new() -> Self {
        Self {
            buy_tickers: Default::default(),
            sell_tickers: Default::default(),
            // lowest_sell: None, // ask price
            // highest_buy: None, // bid price
        }
    }

    /// Only returns the order of the remainder, because we will use a separate service(?) for actual order queueing.
    /// This function reduces as much as possible to manipulating internal state of the exchange.
    pub(crate) async fn execute_placed_limit_order(
        &self,
        limit: Tokens,
        shares: usize,
        symbol: Symbol,
        action: BuySell,
    ) -> Result<Option<Uuid>, anyhow::Error> {
        // TODO: be able to send to specific users based on... GUID?
        // TODO: generate and return a transaction receipt

        // matching references:
        // http://web.archive.org/web/20120626161034/http://www.cmegroup.com/confluence/display/EPICSANDBOX/Match+Algorithms
        // https://stackoverflow.com/questions/13112062/which-are-the-order-matching-algorithms-most-commonly-used-by-electronic-financi
        // https://stackoverflow.com/questions/15603315/efficient-data-structures-for-data-matching
        // https://corporatefinanceinstitute.com/resources/capital-markets/matching-orders/
        let execution_ticker = match action {
            BuySell::Sell => &self.buy_tickers,
            BuySell::Buy => &self.sell_tickers,
        }
        .read()
        .await;

        let mut execution_tree = execution_ticker.get(&symbol).unwrap().write().await;
        let query_limit = match action {
            // FIXME: buy tree contains buy orders, so when querying from sell limit, (+limit, +infinity)
            // sell tree econtains sell orders, so we want to see (-limit, +infinity)
            BuySell::Buy => limit,
            BuySell::Sell => -limit,
        };
        // TODO: fast path to place order if none exist? should not be needed as it is a one-time edge case?

        {
            // FIXME: reverse the ordering on one of the execution trees so we can make this fully generic
            let execution_tree_iter = execution_tree.range_mut((Included(query_limit), Unbounded));

            let mut shares = shares;
            let mut _transferred_tokens = 0_usize;

            'outer: for (&limit_price, limit) in execution_tree_iter {
                let order_iter = limit.orders.iter_mut();
                for order in order_iter {
                    if shares > 0 {
                        // TODO: verify that the buyer still has money (when copying over to buy side, make sure the buyer actually has money)
                        // TODO: implement collateral so that we don't need to do the above per round
                        let transferred_shares = min(order.shares, shares);

                        shares -= transferred_shares;
                        order.shares -= transferred_shares;
                        _transferred_tokens += transferred_shares * (limit_price as usize);

                        // TODO: transfer money from buyer to seller
                        if order.shares == 0 {
                            // TODO: delete more elegantly, possibly using by refactoring order loop to retain_mut? Might have to use .all() instead to short-circuit.
                            order.live = false;
                        }
                    } else {
                        break 'outer;
                    }
                }
                limit.orders.retain(|order| order.live); // TODO: terminate this upon seeing the first True value (because of in-order iteration)
            }
            execution_tree.retain(|_, limit| !limit.orders.is_empty()); // FIXME: see if in practice, this check takes up more time than just iterating over dead limits
        }

        // FIXME: beware deadlock? refactor by grabbing both locks on entrance
        if shares > 0 {
            let standing_order_id = match action {
                BuySell::Buy => &self.buy_tickers,
                BuySell::Sell => &self.sell_tickers,
            }
            .read()
            .await
            .get(&symbol)
            .expect("Symbol should have been inserted upon initialization!")
            .write()
            .await
            .entry(query_limit)
            .or_insert(Limit::new(limit, symbol, 0))
            .insert_order(shares);

            Ok(Some(standing_order_id))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn cancel_order(&self) -> Result<(), anyhow::Error> {
        todo!()
    }
}

#[tonic::async_trait]
impl exchange_server::Exchange for LimitBook {
    async fn submit_limit_order(
        &self,
        req: Request<SubmitLimitOrderRequest>,
    ) -> Result<Response<SubmitLimitOrderReply>, Status> {
        let req = req.into_inner();
        let (limit, shares, ticker) = (req.limit, req.shares as usize, req.ticker);
        let action = match req.action {
            0 => BuySell::Buy,
            1 => BuySell::Sell,
            _ => panic!("WTF how did you get here??"),
        };
        match self
            .execute_placed_limit_order(limit, shares, ticker, action)
            .await
        {
            Ok(order_id) => Ok(Response::new(SubmitLimitOrderReply {
                order_id: order_id.map(|id| id.to_string()),
                message: None,
            })),
            Err(e) => Err(Status::from_error(e.into())),
        }
    }

    async fn cancel_order(
        &self,
        req: Request<CancelOrderRequest>,
    ) -> Result<Response<CancelOrderReply>, Status> {
        match self.cancel_order() {
            _ => todo!(),
        }
    }

    async fn submit_market_order(
        &self,
        req: Request<SubmitMarketOrderRequest>,
    ) -> Result<Response<SubmitMarketOrderReply>, Status> {
        todo!()
    }
}

// TODO: implement LMAX Disruptor architecture
