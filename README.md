# StockFighter

Complete Rust API for [StockFighter](https://www.stockfighter.io)

Find it here: https://crates.io/crates/stockfighter

**Use latest nightly: `multirust run nightly cargo build`**

Make sure to set $STOCKFIGHTER_API_KEY.

## Intro

Simple to get started:

```rust
extern crate stockfighter;

use stockfighter::*;

fn main() {
    let sf = StockFighter::new();
    println!("{:#?}",sf.venue_heartbeat(Venue("TESTEX".to_string())));
}
```

Also a wrapper for the WebSockets endpoints!:

```rust
#[macro_use] extern crate stockfighter;
extern crate serde_json;
extern crate websocket;

use stockfighter::*;
use websocket::{Message, Receiver};
use websocket::message::Type;
use websocket::ws::util::bytes_to_string;

fn main() {
  let sf = StockFighter::new();
  let account = ""; 
  let venue   = "";
   // Open quotes Web Socket
  let mut one = quotes_ws!(Account(account.to_string()), Venue(venue.to_string()));
  for message in one.incoming_messages() {
    let message: Message = match message 
      { Ok(message) => message, Err(e) => { println!("Error: {:?}", e); break; }};
	match message.opcode {
      Type::Text => { match serde_json::from_str::<QuoteWS>(&bytes_to_string(&*message.payload).unwrap()) {
        Ok(o) => println!("{:#?}",Ok(o)), // If Quote parses correctly, print it.
        Err(_) => println!("{:#?}",       // If it doesn't, try parsing it as an ErrMsg.
                           serde_json::from_str::<ErrMsg>(&bytes_to_string(&*message.payload).unwrap())) }}
        _ => () }}}

```
## Functions, types, and macros

```rust
// Funcs

impl StockFighter {

  pub fn new() -> StockFighter {..}

  pub fn api_heartbeat(&self) -> HyperResult<ApiHeartbeat> {..}

  pub fn venue_heartbeat(&self, venue: Venue) -> HyperResult<VenueHeartbeat> {..}

  pub fn stocks_on_venue(&self, venue: Venue) -> HyperResult<StocksOnVenue> {..}

  pub fn orderbook(&self, venue: Venue, stock: Symbol) -> HyperResult<OrderbookForAStock> {..}

  pub fn new_order(&self, acc: Account, venue: Venue, stock: Symbol,
                   price: Price, qty: Qty, dir: Direction, order_type: OrderType)
                   -> HyperResult<NewOrderForAStock> {..}

  pub fn quote(&self, venue: Venue, stock: Symbol) -> HyperResult<QuoteForAStock> {..}

  pub fn status_for_existing_order(&self, id: OrderId, venue: Venue, stock: Symbol)
                                   -> HyperResult<StatusForAnExistingOrder> {..}

  pub fn cancel_order(&self, venue: Venue, stock: Symbol, order: OrderId)
                      -> HyperResult<CancelAnOrder> {..}

  pub fn status_for_all_orders(&self, venue: Venue, acc: Account)
                              -> HyperResult<StatusForAllOrders> {..}

  pub fn status_for_all_orders_in_a_stock(&self, venue: Venue, acc: Account, stock: Symbol)
                                          -> HyperResult<StatusForAllOrdersInAStock> {..} }


// Types

pub type HyperResult<T> = Result<Result<T,hyper::status::StatusCode>,hyper::error::Error>;


// Enums

pub enum VenueHeartbeat { R200(VenueOk), R500(ErrMsg), R404(ErrMsg) }

pub enum StocksOnVenue              { R200(Stocks),    R404(ErrMsg) }
pub enum OrderbookForAStock         { R200(Orderbook), R404(ErrMsg) }
pub enum NewOrderForAStock          { R200(Order),     R404(ErrMsg), R200Err(ErrMsg) }
pub enum QuoteForAStock             { R200(Quote),     R404(ErrMsg) }

pub enum StatusForAnExistingOrder   { R200(Order),     R401(ErrMsg) }
pub enum CancelAnOrder              { R200(Order),     R401(ErrMsg) }
pub enum StatusForAllOrders         { R200(Status),    R401(ErrMsg) }
pub enum StatusForAllOrdersInAStock { R200(Status),    R401(ErrMsg) }

pub enum Direction { Buy, Sell }

pub enum OrderType { Limit, Market, FillOrKill, ImmediateOrCancel }


// Structs

pub struct StockFighter { api_key: String, client: Client }

pub struct ApiHeartbeat(pub ErrMsg);

pub struct ErrMsg { pub ok: bool, pub error: String }

pub struct VenueOk { ok: bool, venue: Venue }


pub struct   Qty(pub usize);
pub struct Price(pub usize);

pub struct   Venue(pub String);
pub struct  Symbol(pub String);
pub struct Account(pub String);

pub struct DTUTC(pub DateTime<UTC>);


pub struct Stocks { pub ok: bool, pub symbols: Vec<SymbolName> }

pub struct SymbolName { pub name: String, pub symbol: Symbol }


pub struct Orderbook {
  pub ok:     bool,
  pub venue:  Venue,
  pub symbol: Symbol,
  pub bids:   Bids,
  pub asks:   Asks,
  pub ts:     DTUTC }

pub struct Bids(pub Vec<Position>);
pub struct Asks(pub Vec<Position>);

pub struct Position {
  pub price:  Price,
  pub qty:    Qty,
  pub is_buy: IsBuy }

pub struct IsBuy(pub bool);


pub struct Order {
  pub ok:           bool,
  pub symbol:       Symbol,
  pub venue:        Venue,
  pub direction:    Direction,
  pub original_qty: OriginalQty,
  pub qty:          Qty,
  pub price:        Price,
  pub order_type:   OrderType,
  pub id:           OrderId,
  pub account:      Account,
  pub ts:           DTUTC,
  pub fills:        Vec<Fill>,
  pub total_filled: TotalFilled,
  pub open:         OrderOpen }

pub struct OriginalQty(pub usize);
pub struct     OrderId(pub usize);
pub struct TotalFilled(pub usize);

pub struct OrderOpen(pub bool);

pub struct Fill { pub price: Price, pub qty: Qty, pub ts: DTUTC }


pub struct NewOrder {
  pub account:    Account,
  pub venue:      Venue,
  pub stock:      Symbol,
  pub qty:        Qty,
  pub price:      Price,
  pub direction:  Direction,
  pub order_type: OrderType }


pub struct Quote {
  pub ok:         Option<bool>,
  pub symbol:     Symbol,
  pub venue:      Venue,
  pub bid:        Option<Bid>,
  pub ask:        Option<Ask>,
  pub bid_size:   Option<BidSize>,
  pub ask_size:   Option<AskSize>,
  pub bid_depth:  Option<BidDepth>,
  pub ask_depth:  Option<AskDepth>,
  pub last_size:  Option<LastSize>,
  pub last_trade: Option<DTUTC>,
  pub quote_time: Option<DTUTC> }

pub struct      Bid(pub usize);
pub struct      Ask(pub usize);
pub struct  BidSize(pub usize);
pub struct  AskSize(pub usize);
pub struct BidDepth(pub usize);
pub struct AskDepth(pub usize);
pub struct     Last(pub usize);
pub struct LastSize(pub usize);


pub struct Status {
  pub ok:     bool,
  pub venue:  Venue,
  pub orders: Vec<Order> }


pub struct QuoteWS { pub ok: bool, pub quote: Quote }

pub struct FillsWS {
  pub ok:                bool,
  pub account:           Account,
  pub venue:             Venue,
  pub symbol:            Symbol,
  pub order:             Order,
  pub standing_id:       StandingId,
  pub incoming_id:       IncomingId,
  pub price:             Price,
  pub filled:            Filled,
  pub filled_at:         DTUTC,
  pub standing_complete: StandingComplete,
  pub incoming_complete: IncomingComplete }

pub struct StandingId(pub usize);
pub struct IncomingId(pub usize);

pub struct Filled(pub usize);

pub struct StandingComplete(pub bool);
pub struct IncomingComplete(pub bool);


// Macros

macro_rules! quotes_ws { ($acc:expr, $venue:expr) => {..} }

macro_rules! quotes_stock_ws { ($acc:expr, $venue:expr, $stock:expr) => {..} }

macro_rules! fills_ws { ($acc:expr, $venue:expr) => {..} }

macro_rules! fills_stock_ws { ($acc:expr, $venue:expr, $stock:expr) => {..} }
```

## Newtypes

All types that have a similar underlying representation but are semantically distinct are wrapped in newtypes.

Newtypes are erased at compile time and exist solely to prevent the library user from
accidentally passing incorrect arguments, such as passing an OrderId into a Price argument.

All newtypes also implement `Deref`, `DerefMut`, `From`, and `Into`.

## Newtypes 2

They also prevent operations between non-similar types, such as multiplying a Price and a Qty.

If you are sure that is what you want to do, you can simply access the underlying data with tuple syntax.

For example: 

```rust
let price_times_qty = price.0 * qty.0;
```
## Newtypes 3

All of the newtypes also implement operator overloading where applicable, so you
do not have to unwrap and rewrap newtypes to do operations on their underlying values.

For example: 

```rust
let multiplied_prices = Price(3241) * Price(1748);
```
## Response Type

All requests return a HyperRequest (if everything goes well) which is constructed with two Oks, which is defined by this type:

```rust
pub type HyperResult<T> = Result<Result<T,hyper::status::StatusCode>,hyper::error::Error>;
```

The reason for this `Ok(Ok(T))` type is that the first layer is for the request itself and if something goes wrong with
the request itself and an `Err(hyper::error::Error)` will be returned. 

If the request goes fine, but the API returns a status that is not anticipated, then it will return the unexpected status code:
`Ok(Err(hyper::status::StatusCode))`

Most of the time both of these will go just fine and it will return `Ok(Ok(RX0X(T)))`, where `RX0X` is some status code, usually
`R200` for success, or `R401`, `R404`, or `R500` for some error case.

## Response Type 2

The main types that are returned as responses are `VenueOk`, `ErrMsg`, `Stocks`, `Orderbook`, `Order`, `Quote`, and `Status.`

The Websocket response types are `QuoteWS` and `FillsWS`.

Here's a full example of a response:

```rust
Ok(
    Ok(
        R200(
            Order {
                ok: true,
                symbol: Symbol(
                    "CWI"
                ),
                venue: Venue(
                    "CSJBEX"
                ),
                direction: Sell,
                original_qty: OriginalQty(
                    10
                ),
                qty: Qty(
                    0
                ),
                price: Price(
                    6000
                ),
                order_type: Limit,
                id: OrderId(
                    1834
                ),
                account: Account(
                    "BFB42332882"
                ),
                ts: DTUTC(
                    "2016-04-14T14:18:28.579888777Z"
                ),
                fills: [
                    Fill {
                        price: Price(
                            6120
                        ),
                        qty: Qty(
                            10
                        ),
                        ts: DTUTC(
                            "2016-04-14T14:18:28.579891467Z"
                        )
                    }
                ],
                total_filled: TotalFilled(
                    10
                ),
                open: OrderOpen(
                    false
                )
            }
        )
    )
)
```
