// have everyone create GUIDs https://docs.rs/chrono/latest/chrono/struct.DateTime.html#method.timestamp
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
pub enum KafkaPayload {
    Send{
        key: String,
        msg: u32
    },
    SendOk{
        offset: u32
    },
    Poll{
        offsets: HashMap<String, u32>
    },
    PollOk{
        msgs: HashMap<String, Vec<(u32,u32)>>
    },
    CommitOffsets{
        offsets: HashMap<String, u32>
    },
    CommitOffsetsOk,
    ListCommittedOffsets{
        keys: Vec<String>
    },
    ListCommittedOffsetsOk{
        offsets: HashMap<String, u32>
    },
    Sync{
        msg: u32,
        offset: u32
    },
    SyncOk
} //send messages to lin-kv to get/set id values

struct KafkaNode{
    name: String,
    id: u32,
    messages: HashMap<String, Vec<(u32, u32)>>,
    offsets: HashMap<String, u32>
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

    init_response.send_stdout();


    let mut sys_node = KafkaNode {
        name: String::new(),
        id: 0,
        messages: HashMap::new(),
        offsets: HashMap::new()
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

    for line in stdin {
        let line = line.context("stdin read failed");
        let request: Message<KafkaPayload>  = serde_json::from_str(
            &line.expect("")
        ).context("")?;
        let node_access = Arc::clone(&node);
        let mut s_node = node_access.lock().unwrap();
        match request.body.body {
            KafkaPayload::Send { key, msg } => {
                let offset = s_node.id;
                if !s_node.messages.contains_key(&key) {
                    s_node.messages.insert(key.clone(), Vec::new());
                }
                s_node.messages.get_mut(&key).unwrap().push((offset,msg));
                let response = Message {
                    src: request.dest,
                    dest: request.src,
                    body: Payload {
                        msg_id: Some(s_node.id),
                        in_reply_to: request.body.msg_id,
                        body: KafkaPayload::SendOk { offset:  offset}
                    }
                };
                s_node.id += 1;
                drop(s_node);
                response.send_stdout();
            }
            KafkaPayload::Poll { offsets } => {
                let mut msgs: HashMap<String, Vec<(u32,u32)>> = HashMap::new();
                
                msgs.extend(offsets.into_iter()
                    .filter(|(key,_)|s_node.messages.contains_key(key))
                    .map(|(key,offset)|{
                    (key.clone(),s_node.messages.get(&key).unwrap().clone().into_iter()
                        .filter(|x|{x.0 >= offset}).into_iter().collect())
                }));

                let response = Message {
                    src: request.dest,
                    dest: request.src,
                    body: Payload {
                        msg_id: Some(s_node.id),
                        in_reply_to: request.body.msg_id,
                        body: KafkaPayload::PollOk { msgs: msgs }
                    }
                };
                s_node.id += 1;
                drop(s_node);
                response.send_stdout();
            }
            KafkaPayload::CommitOffsets { offsets } => {
                for (key,offset) in offsets {
                    s_node.offsets.insert(key, offset);
                }

                let response = Message {
                    src: request.dest,
                    dest: request.src,
                    body: Payload {
                        msg_id: Some(s_node.id),
                        in_reply_to: request.body.msg_id,
                        body: KafkaPayload::CommitOffsetsOk
                    }
                };
                s_node.id += 1;
                drop(s_node);
                response.send_stdout();
            }
            KafkaPayload::ListCommittedOffsets { keys } => {
                let mut committed_messages = HashMap::new();
                for key in keys {
                    if s_node.messages.contains_key(&key) && s_node.offsets.contains_key(&key)  {
                        let committed = s_node.offsets.get(&key).unwrap();
                        committed_messages.insert(key, *committed);
                    }
                }

                let response = Message {
                    src: request.dest,
                    dest: request.src,
                    body: Payload {
                        msg_id: Some(s_node.id),
                        in_reply_to: request.body.msg_id,
                        body: KafkaPayload::ListCommittedOffsetsOk { offsets: committed_messages }
                    }
                };
                s_node.id += 1;
                drop(s_node);
                response.send_stdout();
            }
            _ => panic!()
        };
    }
    let mut end = should_end.lock().unwrap();
    *end = true;
    Ok(())
}