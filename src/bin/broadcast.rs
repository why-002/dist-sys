use dist_sys::*;
use anyhow::Context;
use std::collections::{HashMap, HashSet};
use std::{io, error, thread::{*,self}};
use std::io::{BufRead, Write};
use std::sync::{Arc, Mutex};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{time, vec};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum BroadcastPayload {
    Broadcast{
        message: u32
    },
    BroadcastOk,
    Read,
    ReadOk{
        messages: Vec<u32>
    },
    Topology{
        topology: HashMap<String, Vec<String>>
    },
    TopologyOk,
    Gossip{
        messages: Vec<u32>
    },
    GossipOk{
        messages: Vec<u32>
    }
}

struct BroadcastNode{
    name: String,
    id: u32,
    messages: HashSet<u32>,
    neighbors: Vec<String>
}


fn main() -> anyhow::Result<()>{
    let stdin = io::stdin().lock();
    let mut stdin = stdin.lines();
    let mut stdout = io::stdout().lock();
    let init_msg: Message<InitPayload> = serde_json::from_str(
        &stdin
            .next()
            .expect("no init message received")
            .context("failed to read init message from stdin")?,
    )
    .context("init message could not be deserialized")?;
    
    let init_response =  Message {
        src: init_msg.dest,
        dest: init_msg.src,
        body: Payload {
            msg_id: None,
            in_reply_to: init_msg.body.msg_id,
            body: InitPayload::InitOk
        }
    };

    let serialized = serde_json::to_writer(&mut stdout, &init_response).context("serialize response message");
    stdout.write_all(b"\n").context("trailing newline");
    drop(stdout);


    let mut sys_node = BroadcastNode {
        name: String::new(),
        id: 0,
        messages: HashSet::new(),
        neighbors: Vec::new()
    };

    match init_msg.body.body {
        InitPayload::Init { node_id, node_ids } => {
            sys_node.name = node_id;
        }
        InitPayload::InitOk => {}
    };


    sys_node.id += 1;
    let node = Arc::new(Mutex::new(sys_node));
    let node_ref = Arc::clone(&node);
    let should_end = Arc::new(Mutex::new(false));
    let should_end_ref = Arc::clone(&should_end);

    let handle = thread::spawn(move || {
        let s_node_ref = Arc::clone(&node_ref);
        let s_node = s_node_ref.lock().unwrap();
        let name = String::to_owned(&s_node.name);
        drop(s_node);
        loop{
            sleep(time::Duration::from_millis(50));
            let s_node = s_node_ref.lock().unwrap();
            let messages = HashSet::to_owned(&s_node.messages);
            let neighbors = Vec::to_owned(&s_node.neighbors);
            drop(s_node);
            for n in neighbors.iter() {
                let gossip_message = Message {
                    src: String::to_owned(&name),
                    dest: n.to_string(),
                    body: Payload { 
                        msg_id: None, 
                        in_reply_to: None, 
                        body: BroadcastPayload::Gossip { messages: messages.clone().into_iter().collect() } 
                    }
                };
                let mut stdout = io::stdout().lock();
                serde_json::to_writer(&mut stdout, &gossip_message);
                stdout.write_all(b"\n").context("newline");
            }
            let should_end = Arc::clone(&should_end_ref);
            let end = should_end.lock().unwrap();
            if *end {
                break;
            }
        }
    });

    for line in stdin {
        let line = line.context("stdin read failed");
        let request: Message<BroadcastPayload>  = serde_json::from_str(
            &line.expect("")
        ).context("")?;
        let node_access = Arc::clone(&node);
        let mut s_node = node_access.lock().unwrap();

        match request.body.body {
            BroadcastPayload::Broadcast { message } => {
                s_node.messages.insert(message);
                let response = Message {
                    src: request.dest,
                    dest: request.src,
                    body: Payload {
                        msg_id: Some(s_node.id),
                        in_reply_to: request.body.msg_id,
                        body: BroadcastPayload::BroadcastOk
                    }
                };
                s_node.id += 1;
                drop(s_node);
                let mut stdout = io::stdout().lock();
                serde_json::to_writer(&mut stdout, &response);
                stdout.write_all(b"\n").context("newline");
            }
            BroadcastPayload::Read => {
                let response = Message {
                    src: request.dest,
                    dest: request.src,
                    body: Payload {
                        msg_id: Some(s_node.id),
                        in_reply_to: request.body.msg_id,
                        body: BroadcastPayload::ReadOk { messages: s_node.messages.clone().into_iter().collect() }
                    }
                };
                s_node.id += 1;
                drop(s_node);
                let mut stdout = io::stdout().lock();
                serde_json::to_writer(&mut stdout, &response);
                stdout.write_all(b"\n").context("newline");
            }
            BroadcastPayload::Topology { topology } => {
                s_node.neighbors = Vec::to_owned(topology.get(&s_node.name).unwrap());
                let response = Message {
                    src: request.dest,
                    dest: request.src,
                    body: Payload {
                        msg_id: Some(s_node.id),
                        in_reply_to: request.body.msg_id,
                        body: BroadcastPayload::TopologyOk
                    }
                };
                s_node.id += 1;
                drop(s_node);
                let mut stdout = io::stdout().lock();
                serde_json::to_writer(&mut stdout, &response);
                stdout.write_all(b"\n").context("newline");
            }
            BroadcastPayload::Gossip { messages } => {
                for m in messages.iter(){
                    s_node.messages.insert(*m);
                }
                let response = Message{
                    src: request.dest,
                    dest: request.src,
                    body: Payload {
                        msg_id: None,
                        in_reply_to: None,
                        body: BroadcastPayload::GossipOk { messages: s_node.messages.clone().into_iter().collect() }
                    }
                };
                s_node.id += 1;
                drop(s_node);
                let mut stdout = io::stdout().lock();
                serde_json::to_writer(&mut stdout, &response);
                stdout.write_all(b"\n").context("newline");
            }
            BroadcastPayload::GossipOk { messages } => {
                for m in messages.iter(){
                    s_node.messages.insert(*m);
                }
            }
            _ => panic!()
        };
    }
    let mut end = should_end.lock().unwrap();
    *end = true;
    handle.join();
    Ok(())
}