use dist_sys::*;
use anyhow::Context;
use std::io;
use std::io::{BufRead, Write};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EchoRequest{
    #[serde(rename = "type")]
    typ: String,
    echo: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoResponse {
    #[serde(rename = "type")]
    typ: EchoPayload,
    echo: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EchoPayload {
    EchoOk
}


fn main() -> anyhow::Result<()>{
    let stdin = io::stdin().lock();
    let mut stdin = stdin.lines();
    let mut stdout = io::stdout().lock();
    let init_msg: Message<InitRequest> = serde_json::from_str(
        &stdin
            .next()
            .expect("no init message received")
            .context("failed to read init message from stdin")?,
    )
    .context("init message could not be deserialized")?;
    
    let mut sys_node = Node {
        name: init_msg.body.body.node_id,
        id: 0
    };

    let init_response =  Message {
        src: init_msg.dest,
        dest: init_msg.src,
        body: Payload {
            msg_id: None,
            in_reply_to: init_msg.body.msg_id,
            body: InitResponse{
                typ: InitPayload::InitOk
            }
        }
    };
    sys_node.id += 1;
    let serialized = serde_json::to_writer(&mut stdout, &init_response).context("serialize response message");
    stdout.write_all(b"\n").context("trailing newline");
    for line in stdin {
        let line = line.context("stdin read failed");
        let request: Message<EchoRequest>  = serde_json::from_str(
            &line.expect("")
        ).context("")?;
        let response  = Message {
            src: request.dest,
            dest: request.src,
            body: Payload { 
                msg_id: Some(sys_node.id),
                in_reply_to: request.body.msg_id, 
                body: EchoResponse{
                typ: EchoPayload::EchoOk,
                echo: request.body.body.echo
            }}
        };
        sys_node.id += 1;
        serde_json::to_writer(&mut stdout, &response);
        stdout.write_all(b"\n").context("newline");
    }
    Ok(())
}