use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;
use tokio::task;
use watchexec::event::Tag;
use watchexec::{
    action::Action,
    config::{InitConfig, RuntimeConfig},
    error::ReconfigError,
    event::filekind::FileEventKind,
    event::Event,
    Watchexec,
};

//use crate::reloader::Reload;
use crate::reloader::{self, Message};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct ReloaderSession {
    pub id: usize,
    pub hb: Instant,
    pub addr: Addr<reloader::ReloadServer>,
}

impl ReloaderSession {
    /// helper method that sends ping to client every 5 seconds (HEARTBEAT_INTERVAL).
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // notify chat server
                //act.addr.do_send(reloader::Disconnect { id: act.id });

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

struct MyHandler;

//impl watchexec::Handler<Action> for MyHandler {
//    fn handle(&mut self, _data: Action) -> Result<(), Box<dyn std::error::Error>> {self.handle(_data)}
//}

//impl Actor for ReloaderSession {
//    type Context = ws::WebsocketContext<Self>;
//
//    fn started(&mut self, ctx: &mut Self::Context) {
//        self.hb(ctx);
//        log::info!("Started Actor");
//        let addr = ctx.address(); //which is right?
//                                  //let addr = self.addr.clone();
//
//        let init = InitConfig::default();
//        let mut runtime = RuntimeConfig::default();
//        let filter = |evt: &Event| -> bool {
//            evt.tags.iter().any(|t| {
//                matches!(
//                    t,
//                    &Tag::FileEventKind(FileEventKind::Create(_))
//                        | &Tag::FileEventKind(FileEventKind::Modify(_))
//                        | &Tag::FileEventKind(FileEventKind::Remove(_))
//                )
//            })
//        };
//        runtime.pathset(["trees"]);
//        runtime.on_action(move |action: Action| {
//            let addr = addr.clone();
//            async move {
//                for event in action.events.iter() {
//                    if filter(event) {
//                        //log::info!("Detected Changes!");
//                        println!("Detected Changes!");
//                        addr.send(Message::Reload)
//                            //.into_actor(self)
//                            //.then(|_, _, _| fut::ready(()))
//                            //.wait(ctx);
//                    }
//                }
//                Ok::<(), ReconfigError>(())
//            }
//        });
//        let we = Watchexec::new(init, runtime.clone()).unwrap();
//        let _ = we.main();
//    }
//
//    fn stopping(&mut self, _: &mut Self::Context) -> Running {
//        self.addr.do_send(reloader::Disconnect { id: self.id });
//        Running::Stop
//    }
//}

impl Handler<reloader::Message> for ReloaderSession {
    type Result = String;
    fn handle(&mut self, msg: reloader::Message, ctx: &mut Self::Context) -> String {
        match msg {
            Message::Reload => {
                return "reload".to_string();
            }
            Message::Msg(s) => {
                return s;
            }
        };
        //ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ReloaderSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };
        log::debug!("WEBSOCKET MESSAGE: {msg:?}");
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let m = text.trim();
                self.hb = Instant::now();
                //self.addr
                //    .send(Message::Reload)
                //    .into_actor(self)
                //    .then(|res, _, ctx| {
                //        match res {
                //            Ok(()) => {}
                //            _ => println!("Something is wrong"),
                //        }
                //        fut::ready(())
                //    })
                //    .wait(ctx)
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}

impl Actor for ReloaderSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        let addr = ctx.address(); //which is right?
                                  //let addr = self.addr.clone();

        let init = InitConfig::default();
        let mut runtime = RuntimeConfig::default();
        runtime.pathset(["trees"]);
        runtime.on_action(move |action: Action| {
            let addr = addr.clone();
            async move {
                for event in action.events.iter() {
                    log::info!("Detected Changes");
                    addr.send(Message::Reload);
                }
                Ok::<(), ReconfigError>(())
            }
        });
        let we = Watchexec::new(init, runtime.clone()).unwrap();
        log::info!("Reached Here!");
        //std::thread::spawn(move || we.main());
        //tokio::spawn(we.main());
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.addr.do_send(reloader::Disconnect { id: self.id });
        Running::Stop
    }
}