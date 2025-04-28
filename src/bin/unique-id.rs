use std::convert::Infallible;

use gossip_glomers::{Node, RuntimeError, main_loop};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

struct UniqueId;

impl Node for UniqueId {
    type State = ();
    type Request = Request;
    type Response = Response;
    type HandleError = Infallible;

    fn from_init(_state: Self::State, _init: gossip_glomers::Init) -> Self {
        Self
    }

    fn handle(
        &mut self,
        _input: gossip_glomers::Message<Self::Request>,
    ) -> Result<Self::Response, Self::HandleError> {
        Ok(Response::GenerateOk { id: Uuid::new_v4() })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Request {
    Generate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Response {
    GenerateOk { id: Uuid },
}

fn main() -> Result<(), RuntimeError<Infallible>> {
    main_loop::<UniqueId>(())
}
