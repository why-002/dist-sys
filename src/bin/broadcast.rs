use dist_sys::*;
use anyhow::Context;
use std::collections::HashMap;
use std::{io, error};
use std::io::{BufRead, Write};
use std::sync::{Arc, Mutex};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

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
    TopologyOk
}

struct BroadcastNode{
    name: String,
    id: u32,
    messages: Vec<u32>
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


    let mut sys_node = BroadcastNode {
        name: String::new(),
        id: 0,
        messages: Vec::new()
    };

    match init_msg.body.body {
        InitPayload::Init { node_id, node_ids } => {
            sys_node.name = node_id;
        }
        InitPayload::InitOk => {}
    };


    sys_node.id += 1;
    let node = Arc::new(Mutex::new(sys_node));


    for line in stdin {
        let line = line.context("stdin read failed");
        let request: Message<BroadcastPayload>  = serde_json::from_str(
            &line.expect("")
        ).context("")?;
        let node_access = Arc::clone(&node);
        let mut s_node = node_access.lock().unwrap();

        let response = match request.body.body {
            BroadcastPayload::Broadcast { message } => {
                s_node.messages.push(message);
                Message {
                    src: request.dest,
                    dest: request.src,
                    body: Payload {
                        msg_id: Some(s_node.id),
                        in_reply_to: request.body.msg_id,
                        body: BroadcastPayload::BroadcastOk
                    }
                }
            }
            BroadcastPayload::Read => {
                Message {
                    src: request.dest,
                    dest: request.src,
                    body: Payload {
                        msg_id: Some(s_node.id),
                        in_reply_to: request.body.msg_id,
                        body: BroadcastPayload::ReadOk { messages: Vec::to_owned(&s_node.messages) }
                    }
                }
            }
            BroadcastPayload::Topology { topology } => {
                Message {
                    src: request.dest,
                    dest: request.src,
                    body: Payload {
                        msg_id: Some(s_node.id),
                        in_reply_to: request.body.msg_id,
                        body: BroadcastPayload::TopologyOk
                    }
                }
            }
            _ => panic!()
        };

        s_node.id += 1;
        serde_json::to_writer(&mut stdout, &response);
        stdout.write_all(b"\n").context("newline");

    }
    Ok(())
}