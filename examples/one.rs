#[macro_use] extern crate stockfighter;
             extern crate serde_json;
             extern crate ws;
             extern crate carboxyl;

use std::io::prelude::*;
use std::fs::{File,OpenOptions};
use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as MPSCSender;
use ws::{connect,CloseCode,Handler,Message,Handshake,Result,Error,ErrorKind};
use carboxyl::{Sink,Stream};

use stockfighter::*;

enum SFTTEvent { Connect(Stream<QuoteM>), Disconnect }

struct SFTT { tt_sender: MPSCSender<SFTTEvent>, tt_sink: Sink<QuoteM>, f: File }

impl Handler for SFTT {
  fn on_open(&mut self, _: Handshake) -> Result<()> {
    println!("{}","SFTT connected!");
    self.tt_sender.send(SFTTEvent::Connect(self.tt_sink.stream())).map_err(|err| Error::new(
      ErrorKind::Internal, format!("Unable to communicate between threads: {:?}.", err))) }

  fn on_message(&mut self, msg: Message) -> Result<()> {
    if let Err(e) = self.f.write(&msg.clone().into_data()) { println!("{}", e); }
    match serde_json::from_str::<QuoteWS>(&msg.into_text().unwrap()) {
      Ok(o) => { Ok(self.tt_sink.send_async(quote_ws_to_quote_m(o))) },
      Err(e) => { Err(Error::new(ErrorKind::Internal, format!("SFTT on_message: {:?}.", e))) }}}

  fn on_close(&mut self, code: CloseCode, reason: &str) { }

  fn on_error(&mut self, err: Error) {  }}

fn main() {
  let sf = StockFighter::new();
  let acc   = "";
  let venue = "";
  let stock = "";

  let tt_url = format!("wss://api.stockfighter.io/ob/api/ws/{}/venues/{}/tickertape/stocks/{}",acc,venue,stock);

  let (tt_tx, tt_rx) = channel();

  let _ = thread::Builder::new().name("sf_tt".to_owned()).spawn(move || {
    connect(tt_url, |_| { SFTT { tt_sender: tt_tx.clone(),
                                 tt_sink: Sink::new(),
                                 f: OpenOptions::new().write(true).append(true).open("data/orders.txt").unwrap()
    }}).unwrap(); });

  let tt_stream = match tt_rx.recv() { Ok(SFTTEvent::Connect(stream)) => stream, _ => panic!("tt_sink") };

  let _ = thread::Builder::new().name("trade".to_owned()).spawn(move || {
    for event in tt_stream.events() {
      if let Ok(SFTTEvent::Disconnect) = tt_rx.try_recv() { break }
      /* trade! */ }}).unwrap(); }
