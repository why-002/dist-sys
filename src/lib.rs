use anyhow::Context;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{io::{BufRead, StdoutLock, Write}, string};

pub struct Node {
    pub name: String,
    pub id: u32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<Body> {
    pub src: String,
    pub dest: String,
    pub body: Payload<Body>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload<Body>{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<u32>,
    #[serde(flatten)]
    pub body: Body
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum InitPayload {
    Init{
        node_id: String,
        node_ids: Vec<String>
    },
    InitOk
}