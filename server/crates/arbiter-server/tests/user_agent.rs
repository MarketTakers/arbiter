mod common;

use arbiter_proto::proto::UserAgentResponse;
use arbiter_server::actors::user_agent::UserAgentError;
use kameo::{Actor, actor::Recipient, actor::Spawn, prelude::Message};

/// A no-op actor that discards any messages it receives.
#[derive(Actor)]
struct NullSink;

impl Message<Result<UserAgentResponse, UserAgentError>> for NullSink {
    type Reply = ();

    async fn handle(
        &mut self,
        _msg: Result<UserAgentResponse, UserAgentError>,
        _ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
    }
}

/// Creates a `Recipient` that silently discards all messages.
fn null_recipient() -> Recipient<Result<UserAgentResponse, UserAgentError>> {
    let actor_ref = NullSink::spawn(NullSink);
    actor_ref.recipient()
}

#[path = "user_agent/auth.rs"]
mod auth;
#[path = "user_agent/unseal.rs"]
mod unseal;
