use std::collections::{HashMap, HashSet};

use gossip_glomers::{Body, Message, Node, RuntimeError, main_loop};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Payload {
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: HashSet<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

struct Broadcast {
    node: String,
    messages: HashSet<usize>,
    known: HashMap<String, HashSet<usize>>,
    neighborhood: Vec<String>,
    msg_id: usize,

    msg_communicated: HashMap<String, HashSet<usize>>,
}

#[derive(Debug, thiserror::Error)]
#[error("No topology entry for this node")]
struct NoTopology;

impl Node for Broadcast {
    type Payload = Payload;
    type HandleError = NoTopology;
    type State = ();

    fn from_init(_state: Self::State, init: gossip_glomers::Init) -> Self {
        Self {
            node: init.node_id,
            msg_id: 1,
            messages: HashSet::new(),
            known: init
                .node_ids
                .into_iter()
                .map(|n| (n, HashSet::new()))
                .collect(),
            msg_communicated: HashMap::new(),
            neighborhood: Vec::new(),
        }
    }

    fn handle(
        &mut self,
        input: gossip_glomers::Message<Self::Payload>,
    ) -> Result<Vec<Message<Self::Payload>>, Self::HandleError> {
        let out = match input.body.payload {
            Payload::Broadcast { message } => {
                self.messages.insert(message);
                Ok(vec![Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        msg_id: Some(self.msg_id),
                        in_reply_to: input.body.msg_id,
                        payload: Payload::BroadcastOk,
                    },
                }])
            }
            Payload::Read => Ok(vec![Message {
                src: input.dest,
                dest: input.src,
                body: Body {
                    msg_id: Some(self.msg_id),
                    in_reply_to: input.body.msg_id,
                    payload: Payload::ReadOk {
                        messages: self.messages.clone(),
                    },
                },
            }]),
            Payload::Topology { mut topology } => {
                self.neighborhood = topology.remove(&self.node).ok_or(NoTopology)?;
                Ok(vec![Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        msg_id: Some(self.msg_id),
                        in_reply_to: input.body.msg_id,
                        payload: Payload::TopologyOk,
                    },
                }])
            }
            Payload::BroadcastOk | Payload::ReadOk { .. } | Payload::TopologyOk => Ok(vec![]),
        };
        self.msg_id += 1;
        out
    }
}

fn main() -> Result<(), RuntimeError<NoTopology>> {
    main_loop::<Broadcast>(())
}
