use std::io::Write;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<P> {
    pub src: String,
    pub dest: String,
    pub body: Body<P>,
}

#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error("Failed to serialize message")]
    Serialization(
        #[from]
        #[source]
        serde_json::Error,
    ),
    #[error("Failed to write trailing newline")]
    WriteNewline(
        #[from]
        #[source]
        std::io::Error,
    ),
}

impl<P: Serialize> Message<P> {
    pub fn send_to(&self, out: &mut impl Write) -> Result<(), SendError> {
        serde_json::to_writer(&mut *out, self)?;
        out.write_all(b"\n")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body<P> {
    pub msg_id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: P,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum InitPayload {
    Init(Init),
    InitOk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

pub trait Node: Sized {
    type State;
    type Payload: Serialize + for<'de> Deserialize<'de>;
    type HandleError;

    fn from_init(state: Self::State, init: Init) -> Self;

    fn handle(
        &mut self,
        input: Message<Self::Payload>,
    ) -> Result<Vec<Message<Self::Payload>>, Self::HandleError>;
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError<E> {
    #[error("Improper init handshake")]
    BadInit,
    #[error("Failed to get a line from stdin")]
    Stdin(
        #[from]
        #[source]
        std::io::Error,
    ),
    #[error("Serde error")]
    Serde(
        #[from]
        #[source]
        serde_json::Error,
    ),
    #[error("Failed to send response")]
    Send(
        #[from]
        #[source]
        SendError,
    ),
    #[error("Error handling message")]
    Handle(#[source] E),
}

pub fn main_loop<N: Node>(init_state: N::State) -> Result<(), RuntimeError<N::HandleError>> {
    let stdin = std::io::stdin().lock();
    let mut input =
        serde_json::Deserializer::from_reader(stdin).into_iter::<Message<InitPayload>>();
    let mut stdout = std::io::stdout().lock();

    let init_msg = input.next().ok_or(RuntimeError::BadInit)??;

    let InitPayload::Init(init) = init_msg.body.payload else {
        return Err(RuntimeError::BadInit);
    };

    let mut node = N::from_init(init_state, init);

    let reply = Message {
        src: init_msg.dest,
        dest: init_msg.src,
        body: Body {
            msg_id: init_msg.body.msg_id.map(|id| id + 1),
            in_reply_to: init_msg.body.msg_id,
            payload: InitPayload::InitOk,
        },
    };

    reply.send_to(&mut stdout)?;

    drop(input);

    let stdin = std::io::stdin().lock();
    let input = serde_json::Deserializer::from_reader(stdin).into_iter::<Message<N::Payload>>();

    for msg in input {
        let msg = msg?;

        let replies = node.handle(msg).map_err(RuntimeError::Handle)?;
        for reply in replies {
            reply.send_to(&mut stdout)?;
        }
    }

    Ok(())
}
