use std::convert::Infallible;

use gossip_glomers::{Body, Message, Node, RuntimeError, main_loop};

use serde::{Deserialize, Serialize};

struct Echo;

impl Node for Echo {
    type State = ();
    type Payload = EchoPayload;
    type HandleError = Infallible;

    fn from_init(_state: Self::State, _init: gossip_glomers::Init) -> Self {
        Self
    }

    fn handle(
        &mut self,
        input: gossip_glomers::Message<Self::Payload>,
    ) -> Result<Vec<Message<Self::Payload>>, Self::HandleError> {
        let EchoPayload::Echo { echo } = input.body.payload else {
            return Ok(vec![]);
        };
        Ok(vec![Message {
            src: input.dest,
            dest: input.src,
            body: Body {
                msg_id: input.body.msg_id.map(|id| id + 1),
                in_reply_to: input.body.msg_id,
                payload: EchoPayload::EchoOk { echo },
            },
        }])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum EchoPayload {
    Echo { echo: String },
    EchoOk { echo: String },
}

fn main() -> Result<(), RuntimeError<Infallible>> {
    main_loop::<Echo>(())
}
