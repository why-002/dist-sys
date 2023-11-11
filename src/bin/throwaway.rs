use dist_sys::*;
use anyhow::{Context, Ok};
use std::io;
use std::io::{BufRead, Write};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use threadpool::ThreadPool;
use std::sync::{self, Mutex, Arc};

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
    drop(stdout);

    let node = Arc::new(Mutex::new(sys_node));
    let pool = ThreadPool::new(14);
    for line in stdin {
        let n = Arc::clone(&node);
        let line = line.context("stdin read failed");
        pool.execute( move || {
            let request: Message<EchoRequest>  = serde_json::from_str(
                &line.expect("")
            ).unwrap();
            let mut s_node = n.lock().unwrap();
            s_node.id += 1;
            let s = s_node.id;
            drop(s_node);

            let response  = Message {
                src: request.dest,
                dest: request.src,
                body: Payload { 
                    msg_id: Some(s),
                    in_reply_to: request.body.msg_id, 
                    body: EchoResponse{
                    typ: EchoPayload::EchoOk,
                    echo: request.body.body.echo
                }}
            };
            let mut stdout = io::stdout().lock();
            serde_json::to_writer(&mut stdout, &response);
            stdout.write_all(b"\n").context("newline");
        });  
    }
    pool.join();
    Ok(())
}