use alloy::primitives::map::HashMap;
use arbiter_crypto::authn;
use kameo::{error::Infallible, prelude::*};

use crate::{db::DatabasePool, peers::user_agent::{Credentials, UserAgentSession}};

use super::vault::{Vault, events as vault_events};

pub struct Args {
    pub vault: ActorRef<Vault>,
    pub pool: DatabasePool,
}

pub struct UserAgentRegistry {
    vault: ActorRef<Vault>,
    pool: DatabasePool,
    connected: HashMap<Credentials, ActorRef<UserAgentSession>>,
}

impl Message<vault_events::Bootstrapped> for UserAgentRegistry {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: vault_events::Bootstrapped,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        todo!()
    }
}

impl Message<vault_events::Unsealed> for UserAgentRegistry {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: vault_events::Unsealed,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        todo!()
    }
}
impl Actor for UserAgentRegistry {
    type Args = Args;

    type Error = Infallible;

    async fn on_start(args: Self::Args, actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        Ok(Self {
            vault: args.vault,
            pool: args.pool,
            connected: HashMap::default(),
        })
    }

    
}
