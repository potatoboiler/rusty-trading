use anyhow::Result;
use rpc::exchange::exchange_server;
use std::collections::HashMap;
use tonic::transport::Server;

use types::Symbol;

mod limit_order_book;

enum BuySell {
    Buy,
    Sell,
}
// Order types?
// take profit
// stop loss, stop entry
// market order, limmit
// https://www.investopedia.com/trading-order-types-and-processes-4689649

mod types {
    pub(crate) type Tokens = i64; // 10000 Tokens ~ $1 USD
                                  /*
                                      Decoupling event time and processing time
                                  To reduce the lack of certainty with respect to out of order events, the system should decouple event time from processing time. This would mean that the system should avoid relying on system-wide cursors to determine if a post is ‘new’ (and therefore, due for publication). Instead, it should maintain a blog-level cursor: a post will be considered a ‘new’ post, if it bears a timestamp (event time) that is greater than the timestamp of the last post of that blog.
                                  */
    pub(crate) type Timestamp = u64;
    pub(crate) type Id = usize;
    pub(crate) type Symbol = String;

    pub(crate) type Price = Tokens;
}

struct Account {
    owned: HashMap<Symbol, u32>,
}

mod market_book {
    // https://www.investopedia.com/ask/answers/061615/how-companys-share-price-determined.asp
    struct MarketBook {}
}

pub mod rpc {
    pub mod exchange {
        include!("proto/exchange.rs");
    }
}

pub async fn start() -> Result<()> {
    let addr = "127.0.0.1:6969".parse().unwrap();
    let limit_book: limit_order_book::LimitBook = limit_order_book::LimitBook::new();

    // tokio::spawn(async move { todo!() });

    let svc = exchange_server::ExchangeServer::new(limit_book);
    Server::builder().add_service(svc).serve(addr).await?;
    Ok(())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    println!("Hello, world!");
    start().await?;
    Ok(())
}

// Useful links:
// - https://www.investopedia.com/ask/answers/05/buystoplimit.asp
// Order Flow Payments - https://www.sec.gov/news/studies/ordpay.htm ?
// https://www.cmegroup.com/education/courses/things-to-know-before-trading-cme-futures/what-happens-when-you-submit-an-order.html
// https://www.greenwich.com/blog/difference-between-price-makers-and-market-makers - in case i try to simulate a market maker here
// https://www.youtube.com/watch?v=b1e4t2k2KJY
// https://www.investopedia.com/ask/answers/061615/how-companys-share-price-determined.asp
// https://www.sciencedirect.com/science/article/pii/S2214845016301090 - Limit order placement by high-frequency traders 
// https://www.investopedia.com/terms/b/batchtrading.asp
// https://github.com/Kautenja/limit-order-book/blob/master/notes/lob.md



// TODO: create new module for account bookkeeping and possibly for order matching services? and order execution?