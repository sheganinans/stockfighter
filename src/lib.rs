#![feature(custom_derive,plugin)]
#![plugin(serde_macros)]
#[macro_use] extern crate hyper;
             extern crate serde;
             extern crate serde_json;
#[macro_use] extern crate serializable_enum;
             extern crate websocket;
             extern crate chrono;

use chrono::{DateTime,UTC};
use hyper::client::{Client,response};
use hyper::status::StatusCode;
use std::io::Read;
use serde::de::{Deserialize,Deserializer,Visitor,MapVisitor,EnumVisitor,Error};
use serde::ser::{Serialize,Serializer};
use std::ops::{Add,Div,Mul,Rem,Sub};
use serde_json::Value;
use std::collections::{BTreeMap};

header! { (XStarfighterAuthorization, "X-Starfighter-Authorization") => [String] }

impl StockFighter {

  pub fn new() -> StockFighter {
    StockFighter { api_key: env!("STOCKFIGHTER_API_KEY").to_string(), client: Client::new() }}

  pub fn api_heartbeat(&self) -> HyperResult<ApiHeartbeat> { HyperResult (
    match self.client.get("https://api.stockfighter.io/ob/api/heartbeat").send() {
      Err(e) => Err(e), Ok(mut res) => Ok( match res.status {
        StatusCode::Ok => Ok(parse_sf_json(&mut res)),
        status_code => Err(status_code) })})}

  pub fn venue_heartbeat(&self, venue: Venue) -> HyperResult<VenueHeartbeat> { HyperResult (
    match self.client.get(&format!("https://api.stockfighter.io/ob/api/venues/{}/heartbeat",venue.0)).send() {
      Err(e) => Err(e), Ok(mut res) => Ok( match res.status {
        StatusCode::Ok                  => Ok(VenueHeartbeat::R200(parse_sf_json(&mut res))),
        StatusCode::InternalServerError => Ok(VenueHeartbeat::R500(parse_sf_json(&mut res))),
        StatusCode::NotFound            => Ok(VenueHeartbeat::R404(parse_sf_json(&mut res))),
        status_code => Err(status_code) }) })}

  pub fn stocks_on_venue(&self, venue: Venue) -> HyperResult<StocksOnVenue> { HyperResult (
    match self.client.get(&format!("https://api.stockfighter.io/ob/api/venues/{}/stocks",venue.0))
                     .header(XStarfighterAuthorization(self.api_key.clone())).send()
    { Err(e) => Err(e), Ok(mut res) => Ok(
      match res.status {
        StatusCode::Ok       => Ok(StocksOnVenue::R200(parse_sf_json(&mut res))),
        StatusCode::NotFound => Ok(StocksOnVenue::R404(parse_sf_json(&mut res))),
        status_code => Err(status_code) }) })}

  pub fn orderbook(&self, venue: Venue, stock: Symbol) -> HyperResult<OrderbookForAStock> { HyperResult (
    match self.client.get(&format!("https://api.stockfighter.io/ob/api/venues/{}/stocks/{}", venue.0, stock.0))
                     .header(XStarfighterAuthorization(self.api_key.clone())).send()
    { Err(e) => Err(e), Ok(mut res) => Ok(
      match res.status {
        StatusCode::Ok       => Ok(OrderbookForAStock::R200(parse_sf_json(&mut res))),
        StatusCode::NotFound => Ok(OrderbookForAStock::R404(parse_sf_json(&mut res))),
        status_code => Err(status_code) }) })}

  pub fn new_order(&self, acc: Account, venue: Venue, stock: Symbol, price: Price,
                   qty: Qty, dir: Direction, order_type: OrderType) -> HyperResult<NewOrderForAStock> { HyperResult (
    match self.client.post(&format!("https://api.stockfighter.io/ob/api/venues/{}/stocks/{}/orders",venue.0,stock.0))
                     .header(XStarfighterAuthorization(self.api_key.clone()))
                     .body(&serde_json::to_string(&NewOrder { account: acc, venue: venue, stock: stock, price: price,
                                                              qty: qty, direction: dir, order_type: order_type })
                           .unwrap()).send()
    { Err(e) => Err(e), Ok(ref mut res) => Ok(
      match res.status {
        StatusCode::BadRequest => Ok(NewOrderForAStock::R404(parse_sf_json(res))),
        StatusCode::Ok => { Ok({
          let mut body = String::new();
          match res.read_to_string(&mut body) {
            Err(e) => panic!(e), Ok(_) => match serde_json::from_str::<Order>(&body) {
              Ok(o)  => NewOrderForAStock::R200(o),
              Err(_) => NewOrderForAStock::R200Err(serde_json::from_str(&body).unwrap()) }}})},
        status_code => Err(status_code) }) })}

  pub fn quote(&self, venue: Venue, stock: Symbol) -> HyperResult<QuoteForAStock> { HyperResult (
    match self.client.get(&format!("https://api.stockfighter.io/ob/api/venues/{}/stocks/{}/quote", venue.0, stock.0))
                     .header(XStarfighterAuthorization(self.api_key.clone())).send()
    { Err(e) => Err(e), Ok(mut res) => Ok(
      match res.status {
        StatusCode::Ok       => Ok(QuoteForAStock::R200(str_to_quote(&{
          let mut body = String::new();
          match res.read_to_string(&mut body) { Err(e) => panic!(e), Ok(_) => body }}))),
        StatusCode::NotFound => Ok(QuoteForAStock::R404(parse_sf_json(&mut res))),
        status_code => Err(status_code) }) })}

  pub fn status_for_existing_order(&self, id: OrderId, venue: Venue, stock: Symbol)
                                   -> HyperResult<StatusForAnExistingOrder> { HyperResult (
    match self.client.get(&format!("https://api.stockfighter.io/ob/api/venues/{}/stocks/{}/orders/{}",
                                   venue.0, stock.0, id.0))
                     .header(XStarfighterAuthorization(self.api_key.clone())).send()
    { Err(e) => Err(e), Ok(mut res) => Ok(
      match res.status {
        StatusCode::Ok           => Ok(StatusForAnExistingOrder::R200(parse_sf_json(&mut res))),
        StatusCode::Unauthorized => Ok(StatusForAnExistingOrder::R401(parse_sf_json(&mut res))),
        status_code => Err(status_code) }) })}

  pub fn cancel_order(&self, venue: Venue, stock: Symbol, order: OrderId) -> HyperResult<CancelAnOrder> { HyperResult (
    match self.client.delete(&format!("https://api.stockfighter.io/ob/api/venues/{}/stocks/{}/orders/{}",
                                      venue.0, stock.0, order.0 ))
                     .header(XStarfighterAuthorization(self.api_key.clone())).send()
    { Err(e) => Err(e), Ok(mut res) => Ok(
      match res.status {
        StatusCode::Ok           => Ok(CancelAnOrder::R200(parse_sf_json(&mut res))),
        StatusCode::Unauthorized => Ok(CancelAnOrder::R401(parse_sf_json(&mut res))),
        status_code => Err(status_code) }) })}

  pub fn status_for_all_orders(&self, venue: Venue, acc: Account) -> HyperResult<StatusForAllOrders> { HyperResult (
    match self.client.get(&format!("https://api.stockfighter.io/ob/api/venues/{}/accounts/{}/orders", venue.0, acc.0))
                     .header(XStarfighterAuthorization(self.api_key.clone())).send()
    { Err(e) => Err(e), Ok(mut res) => Ok(
      match res.status {
        StatusCode::Ok           => Ok(StatusForAllOrders::R200(parse_sf_json(&mut res))),
        StatusCode::Unauthorized => Ok(StatusForAllOrders::R401(parse_sf_json(&mut res))),
        status_code => Err(status_code) }) })}

  pub fn status_for_all_orders_in_a_stock(&self, venue: Venue, acc: Account, stock: Symbol)
                                          -> HyperResult<StatusForAllOrdersInAStock> { HyperResult (
    match self.client.get(&format!("https://api.stockfighter.io/ob/api/venues/{}/accounts/{}/stocks/{}/orders",
                                   venue.0, acc.0, stock.0))
                     .header(XStarfighterAuthorization(self.api_key.clone())).send()
    { Err(e) => Err(e), Ok(mut res) => Ok(
      match res.status {
        StatusCode::Ok           => Ok(StatusForAllOrdersInAStock::R200(parse_sf_json(&mut res))),
        StatusCode::Unauthorized => Ok(StatusForAllOrdersInAStock::R401(parse_sf_json(&mut res))),
        status_code => Err(status_code) }) })}}

pub struct HyperResult<T>(pub Result<Result<T,hyper::status::StatusCode>,hyper::error::Error>);

impl<T> HyperResult<T> {

  pub fn all_ok(self) -> T {
    match self.0 { Err(e) => panic!("{:?}",e), Ok(o) => match o { Err(e) => panic!("{:?}",e), Ok(res) => res }}}}


// Ripped from: https://github.com/arienmalec/newtype_macros
#[macro_export] macro_rules! newtype_derive {
  ($alias:ident($t:ty): ) => { };
  ($alias:ident($t:ty): Deref) => { impl ::std::ops::Deref for $alias {
    type Target = $t;
    fn deref<'a>(&'a self) -> &'a $t { let &$alias(ref v) = self; v }}};
  ($alias:ident($t:ty): DerefMut) => { impl ::std::ops::DerefMut for $alias {
    fn deref_mut<'a>(&'a mut self) -> &'a mut $t { let &mut $alias(ref mut v) = self; v }}};
  ($alias:ident($t:ty): From) => { impl ::std::convert::From<$t> for $alias {
    fn from(v: $t) -> Self { $alias(v) }}};
  ($alias:ident($t:ty): Into) => { impl ::std::convert::Into<$t> for $alias {
    fn into(self) -> $t { let $alias(v) = self; v }}};
  ($alias:ident($t:ty): Display) => { impl ::std::fmt::Display for $alias {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
      let $alias(ref v) = *self; <$t as ::std::fmt::Display>::fmt(v, f) }}};
  ($alias:ident($t:ty): Add) => { impl ::std::ops::Add for $alias {
    type Output = $alias;
    fn add(self, rhs: $alias) -> Self {
      let l = ::std::convert::Into::<$t>::into(self);
      let r = ::std::convert::Into::<$t>::into(rhs);
      ::std::convert::From::<$t>::from(l.add(r)) }}};
  ($alias:ident($t:ty): Sub) => { impl ::std::ops::Sub for $alias {
    type Output = $alias;
    fn sub(self, rhs: $alias) -> Self {
      let l = ::std::convert::Into::<$t>::into(self);
      let r = ::std::convert::Into::<$t>::into(rhs);
      ::std::convert::From::<$t>::from(l.sub(r)) }}};
  ($alias:ident($t:ty): Mul) => { impl ::std::ops::Mul for $alias {
    type Output = $alias;
    fn mul(self, rhs: $alias) -> Self {
      let l = ::std::convert::Into::<$t>::into(self);
      let r = ::std::convert::Into::<$t>::into(rhs);
      ::std::convert::From::<$t>::from(l.mul(r)) }}};
  ($alias:ident($t:ty): Div) => { impl ::std::ops::Div for $alias {
    type Output = $alias;
    fn div(self, rhs: $alias) -> Self {
      let l = ::std::convert::Into::<$t>::into(self);
      let r = ::std::convert::Into::<$t>::into(rhs);
      ::std::convert::From::<$t>::from(l.div(r)) }}};
  ($alias:ident($t:ty): Rem) => { impl ::std::ops::Rem for $alias {
    type Output = $alias;
    fn rem(self, rhs: $alias) -> Self {
      let l = ::std::convert::Into::<$t>::into(self);
      let r = ::std::convert::Into::<$t>::into(rhs);
      ::std::convert::From::<$t>::from(l.rem(r)) }}};
  ($alias:ident($t:ty): Neg) => { impl ::std::ops::Neg for $alias {
    type Output = $alias;
    fn neg(self) -> Self {
      let v = ::std::convert::Into::<$t>::into(self); ::std::convert::From::<$t>::from(-v) }}};
  ($alias:ident($t:ty): $keyword:ident) => { unrecognized derive keyword };
  ($alias:ident($t:ty): $($keyword:ident),*) => { $(newtype_derive!($alias($t): $keyword);)* }; }

// Ripped from: https://github.com/arienmalec/newtype_macros
#[macro_export] macro_rules! newtype {
  ($(#[$meta:meta])* struct $alias:ident($t:ty): $($keyword:ident),*) => {
    $(#[$meta])*
      struct $alias($t);
    $(newtype_derive!($alias($t): $keyword);)* };
  ($(#[$meta:meta])* pub struct $alias:ident(pub $t:ty): $($keyword:ident),*) => {
    $(#[$meta])*
      pub struct $alias(pub $t);
    $(newtype_derive!($alias($t): $keyword);)* };
  ($(#[$meta:meta])* pub struct $alias:ident($t:ty): $($keyword:ident),*) => {
    $(#[$meta])*
      pub struct $alias($t);
    $(newtype_derive!($alias($t): $keyword);)* }; }

#[derive(Debug)] pub enum VenueHeartbeat { R200(VenueOk), R500(ErrMsg), R404(ErrMsg) }

#[derive(Debug)] pub enum StocksOnVenue              { R200(Stocks),    R404(ErrMsg) }
#[derive(Debug)] pub enum OrderbookForAStock         { R200(Orderbook), R404(ErrMsg) }
#[derive(Debug)] pub enum NewOrderForAStock          { R200(Order),     R404(ErrMsg), R200Err(ErrMsg) }
#[derive(Debug)] pub enum QuoteForAStock             { R200(QuoteM),     R404(ErrMsg) }

#[derive(Debug)] pub enum StatusForAnExistingOrder   { R200(Order),     R401(ErrMsg) }
#[derive(Debug)] pub enum CancelAnOrder              { R200(Order),     R401(ErrMsg) }
#[derive(Debug)] pub enum StatusForAllOrders         { R200(Status),    R401(ErrMsg) }
#[derive(Debug)] pub enum StatusForAllOrdersInAStock { R200(Status),    R401(ErrMsg) }

impl VenueHeartbeat {
  pub fn from_200(self) -> VenueOk { match self {            VenueHeartbeat::R200(s) => s, p => panic!("{:#?}", p)}}
  pub fn from_500(self) -> ErrMsg  { match self {            VenueHeartbeat::R500(s) => s, p => panic!("{:#?}", p)}}
  pub fn from_404(self) -> ErrMsg  { match self {            VenueHeartbeat::R404(s) => s, p => panic!("{:#?}", p)}}}

impl StocksOnVenue {
  pub fn from_200(self) -> Stocks { match self {              StocksOnVenue::R200(s) => s, p => panic!("{:#?}", p)}}
  pub fn from_404(self) -> ErrMsg { match self {              StocksOnVenue::R404(s) => s, p => panic!("{:#?}", p)}}}

impl OrderbookForAStock {
  pub fn from_200(self) -> Orderbook { match self {      OrderbookForAStock::R200(s) => s, p => panic!("{:#?}", p)}}
  pub fn from_404(self) -> ErrMsg    { match self {      OrderbookForAStock::R404(s) => s, p => panic!("{:#?}", p)}}}

impl NewOrderForAStock {
  pub fn from_200(self)    -> Order  { match self {    NewOrderForAStock::R200   (s) => s, p => panic!("{:#?}", p)}}
  pub fn from_404(self)    -> ErrMsg { match self {    NewOrderForAStock::R404   (s) => s, p => panic!("{:#?}", p)}}
  pub fn from_200_err(self)-> ErrMsg { match self {    NewOrderForAStock::R200Err(s) => s, p => panic!("{:#?}", p)}}}

impl QuoteForAStock {
  pub fn from_200(self) -> QuoteM { match self {             QuoteForAStock::R200(s) => s, p => panic!("{:#?}", p)}}
  pub fn from_404(self) -> ErrMsg { match self {             QuoteForAStock::R404(s) => s, p => panic!("{:#?}", p)}}}

impl StatusForAnExistingOrder {
  pub fn from_200(self) -> Order  { match self {   StatusForAnExistingOrder::R200(s) => s, p => panic!("{:#?}", p)}}
  pub fn from_401(self) -> ErrMsg { match self {   StatusForAnExistingOrder::R401(s) => s, p => panic!("{:#?}", p)}}}

impl CancelAnOrder {
  pub fn from_200(self) -> Order  { match self {              CancelAnOrder::R200(s) => s, p => panic!("{:#?}", p)}}
  pub fn from_401(self) -> ErrMsg { match self {              CancelAnOrder::R401(s) => s, p => panic!("{:#?}", p)}}}

impl StatusForAllOrders {
  pub fn from_200(self) -> Status { match self {         StatusForAllOrders::R200(s) => s, p => panic!("{:#?}", p)}}
  pub fn from_401(self) -> ErrMsg { match self {         StatusForAllOrders::R401(s) => s, p => panic!("{:#?}", p)}}}

impl StatusForAllOrdersInAStock {
  pub fn from_200(self) -> Status { match self { StatusForAllOrdersInAStock::R200(s) => s, p => panic!("{:#?}", p)}}
  pub fn from_401(self) -> ErrMsg { match self { StatusForAllOrdersInAStock::R401(s) => s, p => panic!("{:#?}", p)}}}

serializable_enum! {
  #[derive(Debug, PartialEq, Eq, Clone)]
  pub enum Direction {
    /// Buy
    Buy,
    /// Sell
    Sell } DirectionVisitor }

impl_as_ref_from_str! { Direction { Buy => "buy", Sell => "sell", } EError::Parse }

serializable_enum! {
  #[derive(Debug, PartialEq, Eq, Clone)]
  pub enum OrderType {
    /// Limit
    Limit,
    /// Market
    Market,
    /// Fill or Kill
    FillOrKill,
    /// Immediate or Cancel
    ImmediateOrCancel } OrderTypeVisitor }

impl_as_ref_from_str! { OrderType { Limit => "limit",
                                    Market => "market",
                                    FillOrKill => "fill-or-kill",
                                    ImmediateOrCancel => "immediate-or-cancel", } EError::Parse }

#[derive(Debug)] pub enum EError { Parse(String) }

impl std::fmt::Display for EError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{:?}", self) }}


pub struct StockFighter { api_key: String, client: Client }

newtype!(#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
         pub struct ApiHeartbeat(pub ErrMsg): Deref, DerefMut, From, Into);

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)] pub struct ErrMsg { pub ok: bool, pub error: String }

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)] pub struct VenueOk { ok: bool, venue: Venue }


newtype!(#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
         pub struct   Qty(pub usize): Deref, DerefMut, From, Into, Display, Add, Sub, Mul, Div, Rem);
newtype!(#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
         pub struct Price(pub usize): Deref, DerefMut, From, Into, Display, Add, Sub, Mul, Div, Rem);

newtype!(#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
         pub struct   Venue(pub String): Deref, DerefMut, From, Into, Display);
newtype!(#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
         pub struct  Symbol(pub String): Deref, DerefMut, From, Into, Display);
newtype!(#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
         pub struct Account(pub String): Deref, DerefMut, From, Into, Display);

newtype!(#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]  pub struct DTUTC(pub DateTime<UTC>): Deref, DerefMut, From, Into);

impl Serialize for DTUTC {
  fn serialize<S: Serializer>(&self, s: &mut S) -> Result<(), S::Error> { s.serialize_unit_struct("DTUTC") }}

impl Deserialize for DTUTC {
  fn deserialize<D: Deserializer>(ds: &mut D) -> Result<DTUTC,D::Error> {
    struct DTUTCVisitor;
    impl Visitor for DTUTCVisitor {
      type Value = DTUTC;
      fn visit_str<E: serde::de::Error>(&mut self, val: &str) -> Result<DTUTC,E> {
        Ok(DTUTC(val.parse::<DateTime<UTC>>().unwrap())) }}
    ds.deserialize_unit_struct("DTUTC",DTUTCVisitor) }}


#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)] pub struct Stocks { pub ok: bool, pub symbols: Vec<SymbolName> }

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)] pub struct SymbolName { pub name: String, pub symbol: Symbol }


#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Orderbook {
  pub ok:     bool,
  pub venue:  Venue,
  pub symbol: Symbol,
  pub bids:   Bids,
  pub asks:   Asks,
  pub ts:     DTUTC }

newtype!(#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
         pub struct Bids(pub Vec<Position>): Deref, DerefMut, From, Into);
newtype!(#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
         pub struct Asks(pub Vec<Position>): Deref, DerefMut, From, Into);

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Position {
  pub price:  Price,
  pub qty:    Qty,
  #[serde(rename="isBuy")]
  pub is_buy: IsBuy }

newtype!(#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
         pub struct IsBuy(pub bool): Deref, DerefMut, From, Into, Display);


#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Order {
  pub ok:           bool,
  pub symbol:       Symbol,
  pub venue:        Venue,
  pub direction:    Direction,
  #[serde(rename="originalQty")]
  pub original_qty: OriginalQty,
  pub qty:          Qty,
  pub price:        Price,
  #[serde(rename="orderType")]
  pub order_type:   OrderType,
  pub id:           OrderId,
  pub account:      Account,
  pub ts:           DTUTC,
  pub fills:        Vec<Fill>,
  #[serde(rename="totalFilled")]
  pub total_filled: TotalFilled,
  pub open:         OrderOpen }

newtype!(#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
         pub struct OriginalQty(pub usize): Deref, DerefMut, From, Into, Display, Add, Sub, Mul, Div, Rem);
newtype!(#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
         pub struct OrderId(pub usize): Deref, DerefMut, From, Into, Display);
newtype!(#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
         pub struct TotalFilled(pub usize): Deref, DerefMut, From, Into, Display, Add, Sub, Mul, Div, Rem);

newtype!(#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
         pub struct OrderOpen(pub bool): Deref, DerefMut, From, Into, Display);

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)] pub struct Fill { pub price: Price, pub qty: Qty, pub ts: DTUTC }


#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct NewOrder {
  pub account:    Account,
  pub venue:      Venue,
  pub stock:      Symbol,
  pub qty:        Qty,
  pub price:      Price,
  pub direction:  Direction,
  #[serde(rename="orderType")]
  pub order_type: OrderType }

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Status {
  pub ok:     bool,
  pub venue:  Venue,
  pub orders: Vec<Order> }

#[derive(Debug, PartialEq, Clone)]
pub struct QuoteM {
  pub symbol:       Symbol,
  pub venue:        Venue,
  pub these_quotes: TheseQuotes,
  pub last:         Last,
  pub last_size:    LastSize,
  pub last_trade:   DateTime<UTC>,
  pub quote_time:   DateTime<UTC> }

#[derive(Debug, PartialEq, Clone)]
pub enum TheseQuotes { ThisBid(BidStruct), ThatAsk(AskStruct), TheseQuotes(BidStruct,AskStruct), Empty }

#[derive(Debug, PartialEq, Clone)]
pub struct BidStruct { bid: Bid, bid_size: BidSize, bid_depth: BidDepth }
#[derive(Debug, PartialEq, Clone)]
pub struct AskStruct { ask: Ask, ask_size: AskSize, ask_depth: AskDepth }

newtype!(#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
         pub struct      Bid(pub u64): Deref, DerefMut, From, Into, Display, Add, Sub, Mul, Div, Rem);
newtype!(#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
         pub struct      Ask(pub u64): Deref, DerefMut, From, Into, Display, Add, Sub, Mul, Div, Rem);
newtype!(#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
         pub struct  BidSize(pub u64): Deref, DerefMut, From, Into, Display, Add, Sub, Mul, Div, Rem);
newtype!(#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
         pub struct  AskSize(pub u64): Deref, DerefMut, From, Into, Display, Add, Sub, Mul, Div, Rem);
newtype!(#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
         pub struct BidDepth(pub u64): Deref, DerefMut, From, Into, Display, Add, Sub, Mul, Div, Rem);
newtype!(#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
         pub struct AskDepth(pub u64): Deref, DerefMut, From, Into, Display, Add, Sub, Mul, Div, Rem);
newtype!(#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
         pub struct     Last(pub u64): Deref, DerefMut, From, Into, Display, Add, Sub, Mul, Div, Rem);
newtype!(#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
         pub struct LastSize(pub u64): Deref, DerefMut, From, Into, Display, Add, Sub, Mul, Div, Rem);

pub fn str_to_quote(s: &str) -> QuoteM {
  let o = serde_json::from_str::<Value>(&s).unwrap();
  let oo = o.as_object().unwrap();
  let q = oo.get("quote").unwrap().as_object().unwrap();
  let quote = match q.get("bid") {
    None => { match q.get("ask") { None => TheseQuotes::Empty, Some(ask) => TheseQuotes::ThatAsk(parse_ask(ask,&q)) }}
              Some(bid) => { match q.get("ask") { None => { TheseQuotes::ThisBid(parse_bid(bid,&q)) },
                                                  Some(ask) =>
                                                  { TheseQuotes::TheseQuotes(parse_bid(bid,&q), parse_ask(ask,&q)) }}}};
  QuoteM {  symbol:       Symbol  (q.get(   "symbol").unwrap().as_string().unwrap().to_string()),
            venue:        Venue   (q.get(    "venue").unwrap().as_string().unwrap().to_string()),
            these_quotes:          quote,
            last:         Last    (q.get(     "last").unwrap().as_u64().unwrap()),
            last_size:    LastSize(q.get( "lastSize").unwrap().as_u64().unwrap()),
            last_trade:            q.get("lastTrade").unwrap().as_string().unwrap().parse::<DateTime<UTC>>().unwrap(),
            quote_time:            q.get("quoteTime").unwrap().as_string().unwrap().parse::<DateTime<UTC>>().unwrap() }}

pub fn parse_bid(bid: &Value, q: &BTreeMap<String, Value>) -> BidStruct {
  BidStruct { bid:       Bid(bid.as_u64().unwrap()),
              bid_size:  BidSize (q.get( "bidSize").unwrap().as_u64().unwrap()),
              bid_depth: BidDepth(q.get("bidDepth").unwrap().as_u64().unwrap()) }}

pub fn parse_ask(ask: &Value, q: &BTreeMap<String, Value>) -> AskStruct {
  AskStruct { ask:       Ask(ask.as_u64().unwrap()),
              ask_size:  AskSize (q.get( "askSize").unwrap().as_u64().unwrap()),
              ask_depth: AskDepth(q.get("askDepth").unwrap().as_u64().unwrap()) }}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct FillsWS {
  pub ok:                bool,
  pub account:           Account,
  pub venue:             Venue,
  pub symbol:            Symbol,
  pub order:             Order,
  #[serde(rename="standingId")]
  pub standing_id:       StandingId,
  #[serde(rename="incomingId")]
  pub incoming_id:       IncomingId,
  pub price:             Price,
  pub filled:            Filled,
  #[serde(rename="filledAt")]
  pub filled_at:         DTUTC,
  #[serde(rename="standingComplete")]
  pub standing_complete: StandingComplete,
  #[serde(rename="incomingComplete")]
  pub incoming_complete: IncomingComplete }

newtype!(#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
         pub struct StandingId(pub usize): Deref, DerefMut, From, Into);
newtype!(#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
         pub struct IncomingId(pub usize): Deref, DerefMut, From, Into);

newtype!(#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
         pub struct Filled(pub usize): Deref, DerefMut, From, Into, Add, Sub, Mul, Div, Rem);

newtype!(#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
         pub struct StandingComplete(pub bool): Deref, DerefMut, From, Into);
newtype!(#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
         pub struct IncomingComplete(pub bool): Deref, DerefMut, From, Into);


fn parse_sf_json<A: serde::Deserialize>(ref mut res: &mut response::Response) -> A {
  let mut body = String::new();
  match res.read_to_string(&mut body) { Err(e) => panic!(e), Ok(_) => serde_json::from_str(&body).unwrap() }}

#[macro_export] macro_rules! quotes_ws {
  ($acc:expr, $venue:expr) => {
 	  { let request = ::websocket::Client::connect(
      ::websocket::client::request::Url::parse(
        &format!("wss://api.stockfighter.io/ob/api/ws/{}/venues/{}/tickertape",
                 ::stockfighter::Account::from($acc), ::stockfighter::Venue::from($venue)))
        .unwrap()).unwrap();
      let response = request.send().unwrap();
      response.validate().unwrap();
      response.begin().split().1 }}}

#[macro_export] macro_rules! quotes_stock_ws {
  ($acc:expr, $venue:expr, $stock:expr) => {
 	  { let request = ::websocket::Client::connect(
      ::websocket::client::request::Url::parse(
        &format!("wss://api.stockfighter.io/ob/api/ws/{}/venues/{}/tickertape/stocks/{}",
                 ::stockfighter::Account::from($acc),
                 ::stockfighter::Venue::from($venue),
                 ::stockfighter::Symbol::from($stock)))
        .unwrap()).unwrap();
      let response = request.send().unwrap();
      response.validate().unwrap();
      response.begin().split().1 }}}

#[macro_export] macro_rules! fills_ws {
  ($acc:expr, $venue:expr) => {
 	  { let request = ::websocket::Client::connect(
      ::websocket::client::request::Url::parse(
        &format!("wss://api.stockfighter.io/ob/api/ws/{}/venues/{}/executions",
                 ::stockfighter::Account::from($acc), ::stockfighter::Venue::from($venue)))
        .unwrap()).unwrap();
      let response = request.send().unwrap();
      response.validate().unwrap();
      response.begin().split().1 }}}

#[macro_export] macro_rules! fills_stock_ws {
  ($acc:expr, $venue:expr, $stock:expr) => {
 	  { let request = ::websocket::Client::connect(
      ::websocket::client::request::Url::parse(
        &format!("wss://api.stockfighter.io/ob/api/ws/{}/venues/{}/executions/stocks/{}",
                 ::stockfighter::Account::from($acc),
                 ::stockfighter::Venue::from($venue),
                 ::stockfighter::Symbol::from($stock)))
        .unwrap()).unwrap();
      let response = request.send().unwrap();
      response.validate().unwrap();
      response.begin().split().1 }}}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_todo() { /* It works, for now! */ }}
