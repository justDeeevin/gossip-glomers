use gossip_glomers::{Node, main_loop};

use serde::{Deserialize, Serialize};

struct Echo;

impl Node for Echo {
    type State = ();
    type Request = EchoRequest;
    type Response = EchoResponse;
    type HandleError = std::convert::Infallible;

    fn from_init(_state: Self::State, _init: gossip_glomers::Init) -> Self {
        Self
    }

    fn handle(
        &mut self,
        input: gossip_glomers::Message<Self::Request>,
    ) -> Result<Self::Response, Self::HandleError> {
        let EchoRequest::Echo { echo } = input.body.payload;
        Ok(EchoResponse::EchoOk { echo })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum EchoRequest {
    Echo { echo: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum EchoResponse {
    EchoOk { echo: String },
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    main_loop::<Echo>(())?;

    Ok(())
}
