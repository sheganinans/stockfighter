#[macro_use] extern crate stockfighter;
extern crate serde_json;
extern crate websocket;

use stockfighter::*;
use websocket::{Message, Receiver};
use websocket::message::Type;
use websocket::ws::util::bytes_to_string;

fn main() {
  let sf = StockFighter::new();
  let account = ""; // Start up a level instance and add the details here.
  let venue   = "";
  let symbol  = "";
  println!("{:#?}",sf.api_heartbeat());
  println!("{:#?}",sf.venue_heartbeat(Venue("TESTEX".to_string()))); // Explicit wrapping with Venue
  println!("{:#?}",sf.stocks_on_venue("TESTEX".to_string().into())); // Wrapping Venue with .into() syntax, use this sparingly.
  println!("{:#?}",sf.orderbook(Venue("TESTEX".to_string()), Symbol("FOOBAR".to_string())));
  println!("{:#?}",sf.quote(Venue(venue.to_string()), Symbol(symbol.to_string())));
  println!("{:#?}",sf.new_order(Account(account.to_string()), Venue(venue.to_string()), Symbol(symbol.to_string()),
                              2900.into(), 10.into(), Direction::Sell, OrderType::Limit));
  println!("{:#?}",sf.status_for_all_orders(Venue(venue.to_string()),Account(account.to_string())));
  let mut one = quotes_ws!(Account(account.to_string()), Venue(venue.to_string())); // Open quotes Web Socket
  for message in one.incoming_messages() {
    let message: Message = match message { Ok(message) => message, Err(e) => { println!("Error: {:?}", e); break; }};
    match message.opcode {
      Type::Text => { match serde_json::from_str::<QuoteWS>(&bytes_to_string(&*message.payload).unwrap()) {
        Ok(o) => println!("{:#?}",Ok(o)), // If Quote parses correctly, print it.
        Err(_) => println!("{:#?}",       // If it doesn't, try parsing it at an ErrMsg.
                           serde_json::from_str::<ErrMsg>(&bytes_to_string(&*message.payload).unwrap())) }}
      _ => () }}}
 
