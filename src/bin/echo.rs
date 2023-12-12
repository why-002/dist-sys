use dist_sys::*;
use anyhow::Context;
use std::io;
use std::io::{BufRead, Write};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum EchoPayload {
    Echo{
        echo: String
    },
    EchoOk{
        echo: String
    }
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
    
    let mut sys_node = Node {
        name: String::new(),
        id: 0
    };

    match init_msg.body.body {
        InitPayload::Init { node_id, node_ids } => {
            sys_node.name = node_id;
        }
        InitPayload::InitOk => {}
    };

    let init_response =  Message {
        src: init_msg.dest,
        dest: init_msg.src,
        body: Payload {
            msg_id: None,
            in_reply_to: init_msg.body.msg_id,
            body: InitPayload::InitOk
        }
    };
    sys_node.id += 1;
    init_response.send_stdout();


    for line in stdin {
        let line = line.context("stdin read failed");
        let request: Message<EchoPayload>  = serde_json::from_str(
            &line.expect("")
        ).context("")?;
        let response = match request.body.body {
            EchoPayload::Echo { echo } => {
                Message {
                    src: request.dest,
                    dest: request.src,
                    body: Payload { 
                        msg_id: Some(sys_node.id),
                        in_reply_to: request.body.msg_id, 
                        body: EchoPayload::EchoOk { echo: echo }
                    }
                }
            }
                EchoPayload::EchoOk { echo } => panic!()
        };
        sys_node.id += 1;
        response.send_stdout();
    }
    Ok(())
}