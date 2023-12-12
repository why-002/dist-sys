
//Have a local cache of the other nodes' values originally intialized to zero, have a background thread gossip/sync
//return the sum of the local cache and the current node's value

use dist_sys::*;
use anyhow::Context;
use std::{io,thread::{*,self}};
use std::io::{BufRead, Write, };
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};
use std::{time, vec};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum GrowPayload {
    Add{
        delta: u32
    },
    AddOk,
    Read,
    ReadOk{
        value: u32
    },
    Gossip{
        value: u32
    }
}

struct GrowNode{
    name: String,
    id: u32,
    value: u32,
    cache: HashMap<String, u32>
}


fn main() -> anyhow::Result<()>{
    let stdin = io::stdin().lock();
    let mut stdin = stdin.lines();
    let init_msg: Message<InitPayload> = serde_json::from_str(
        &stdin
            .next()
            .expect("no init message received")
            .context("failed to read init message from stdin")?,
    )
    .context("init message could not be deserialized")?;
    
    let mut sys_node = GrowNode {
        name: String::new(),
        id: 0,
        value: 0,
        cache: HashMap::new()
    };

    match init_msg.body.body {
        InitPayload::Init { node_id, node_ids } => {
            sys_node.name = node_id;
            for n in node_ids.iter() {
                sys_node.cache.insert((*n).clone(), 0);
                eprintln!("added {} to cache", *n)
            }
        }
        InitPayload::InitOk => {}
    };
    eprint!("Finished Init");
    let init_response =  Message {
        src: init_msg.dest,
        dest: init_msg.src,
        body: Payload {
            msg_id: Some(sys_node.id),
            in_reply_to: init_msg.body.msg_id,
            body: InitPayload::InitOk
        }
    };
    sys_node.id += 1;
    init_response.send_stdout().expect("failed to send init response");



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
            sleep(time::Duration::from_millis(250));
            let s_node = s_node_ref.lock().unwrap();
            let cache:Vec<String> = s_node.cache.clone().into_keys().collect();
            let name = s_node.name.clone();
            let value = s_node.value;
            drop(s_node);
            // for efficency, you should turn this iter into a map, and then grab io to print
            for n in cache.iter() {
                eprintln!("name {}", n);
                if *n != name {
                    let gossip_message = Message {
                        src: String::to_owned(&name),
                        dest: n.to_string(),
                        body: Payload { 
                            msg_id: None,
                            in_reply_to: None, 
                            body: GrowPayload::Gossip { value: value }
                        }
                    };
    
                    gossip_message.send_stdout().expect("Failed to send gossip message");
                }
                
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
        let request: Message<GrowPayload>  = serde_json::from_str(
            &line.expect("")
        ).context("")?;

        let node_access = Arc::clone(&node);
        let mut s_node = node_access.lock().unwrap();
        match request.body.body {
            GrowPayload::Add { delta } => {
                s_node.value += delta;
                let id = s_node.id;
                s_node.id += 1;
                drop(s_node);
                let response = Message {
                    src: request.dest,
                    dest: request.src,
                    body: Payload {
                        msg_id: Some(id),
                        in_reply_to: request.body.msg_id,
                        body: GrowPayload::AddOk
                    } 
                };

                response.send_stdout();
            }
            GrowPayload::Read => {
                let mut total: u32 = s_node.cache.clone().values().into_iter().sum();
                total += s_node.value;
                let id = s_node.id;
                s_node.id += 1;
                drop(s_node);
                let response = Message {
                    src: request.dest,
                    dest: request.src,
                    body: Payload {
                        msg_id: Some(id),
                        in_reply_to: request.body.msg_id,
                        body: GrowPayload::ReadOk { value: total }
                    } 
                };
                
                response.send_stdout();
            }
            GrowPayload::Gossip { value } => {
                s_node.cache.insert(request.src, value);
            }
            _ => panic!()
        };
    }
    let mut end = should_end.lock().unwrap();
    *end = true;
    handle.join();
    Ok(())
}