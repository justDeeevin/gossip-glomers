use std::convert::Infallible;

use gossip_glomers::{Body, Message, Node, RuntimeError, main_loop};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

struct UniqueId;

impl Node for UniqueId {
    type State = ();
    type Payload = Payload;
    type HandleError = Infallible;

    fn from_init(_state: Self::State, _init: gossip_glomers::Init) -> Self {
        Self
    }

    fn handle(
        &mut self,
        input: gossip_glomers::Message<Self::Payload>,
    ) -> Result<Vec<Message<Self::Payload>>, Self::HandleError> {
        Ok(vec![Message {
            src: input.dest,
            dest: input.src,
            body: Body {
                msg_id: input.body.msg_id.map(|id| id + 1),
                in_reply_to: input.body.msg_id,
                payload: Payload::GenerateOk { id: Uuid::new_v4() },
            },
        }])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Payload {
    Generate,
    GenerateOk { id: Uuid },
}

fn main() -> Result<(), RuntimeError<Infallible>> {
    main_loop::<UniqueId>(())
}
