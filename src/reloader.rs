//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available rooms. Peers send messages to other peers in same
//! room through `ChatServer`.

use std::collections::HashMap;

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};

/// Chat server sends this messages to session
#[derive(Message)]
#[rtype(result = "String")]
pub enum Message {
    /// Peer message
    Msg(String),
    Reload,
}

/// Message for chat server communications

/// New chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Debug)]
pub struct ReloadServer {
    sessions: HashMap<usize, Recipient<Message>>,
    rng: ThreadRng,
}

impl ReloadServer {
    pub fn new() -> ReloadServer {
        ReloadServer {
            sessions: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }
}

impl ReloadServer {
    /// Send message to all users in the room
    fn send_message(&self, message: &str) {
        for (id, _) in &self.sessions {
            if let Some(addr) = self.sessions.get(&id) {
                addr.do_send(Message::Msg(message.to_owned()));
            }
        }
    }
}

impl Actor for ReloadServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for ReloadServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");
        // notify all users in same room
        self.send_message("main");

        // register session with random id
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        // auto join session to main room
        //self.rooms.entry("main".to_owned()).or_default().insert(id);

        //let count = self.visitor_count.fetch_add(1, Ordering::SeqCst);
        self.send_message("main");

        // send id back
        id
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ReloadServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        // send message to other users
        self.send_message("Someone disconnected.");
    }
}

impl Handler<Message> for ReloadServer {
    type Result = String;
    fn handle(&mut self, msg: Message, _: &mut Context<Self>) -> String {
        println!("Someone disconnected");

        // send message to other users
        self.send_message("Someone disconnected.");
        return "message".into();
    }
}