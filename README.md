# StockFighter

Complete Rust API for [StockFighter](https://www.stockfighter.io)

# Intro

Simple to get started:

```rustc
extern crate stockfighter;

use stockfighter::*;

fn main() {
    let sf = StockFighter::new();
    println!("{:#?}",sf.venue_heartbeat(Venue("TESTEX".to_string())));
}
```

Also a wrapper for the WebSockets endpoints!:

```rustc
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
  let mut one = quotes_ws!(Account(account.to_string()), Venue(venue.to_string())); // Open quotes Web Socket
  for message in one.incoming_messages() {
    let message: Message = match message { Ok(message) => message, Err(e) => { println!("Error: {:?}", e); break; }};
	match message.opcode {
      Type::Text => { match serde_json::from_str::<QuoteWS>(&bytes_to_string(&*message.payload).unwrap()) {
        Ok(o) => println!("{:#?}",Ok(o)), // If Quote parses correctly, print it.
        Err(_) => println!("{:#?}",       // If it doesn't, try parsing it at an ErrMsg.
                           serde_json::from_str::<ErrMsg>(&bytes_to_string(&*message.payload).unwrap())) }}
        _ => () }}}

```

All types that have a similar underlying representation but are semantically distinct are wrapped in newtypes. 
Newtypes are erased at compile time and exist solely to prevent the library user from
accidentally passing incorrect arguments, such as passing an OrderId into a Price argument.
All newtypes also implement `Deref`, `DerefMut`, `From`, and `Into`.

They also prevent operations between non-similar types, such as multiplying a Price and a Qty.
If you are sure that is what you want to do, you can simply access the underlying data simply with tuple syntax.
For example: `let price_time_qty = price.0 * qty.0;`.

All of the newtypes also implement operator overloading where applicable, so you
do not have to unwrap and rewrap newtypes to do operations on their underlying values.
For example: `let multiplied_prices = Price(3241) * Price(1748);`.

All requests return a HyperRequest (if everything goes well) which is constructed with two Oks, which is defined by this type:

```rustc
pub type HyperResult<T> = Result<Result<T,hyper::status::StatusCode>,hyper::error::Error>;
```
The reason for this `Ok(Ok(T))` type is that the first layer is for the request itself and if something goes wrong with
the request itself and an `Err(hyper::error::Error)` will be returned. 
If the request goes fine, but the API returns a status that is not anticipated, then it will return the unexpected status code:
`Ok(Err(hyper::status::StatusCode))`
Most of the time both of these will go just fine and it will return `Ok(Ok(RX0X(T)))`, where RX0X is some status code, usually
R200 for success, or R401, R404, or R500 for some error case.

The main types that are returned as responses are `VenueOk`, `ErrMsg`, `Stocks`, `Orderbook`, `Order`, `Quote`, and `Status.`

Here's a full example of a response:

```rustc
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
