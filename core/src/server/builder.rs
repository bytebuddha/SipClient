use crate::prelude::*;

#[cfg(feature = "gst")]
use crate::streaming::GStreamerProvider;

use std::collections::HashMap;

pub struct Builder<T> {
    rpc: Option<RpcFuture<T>>,
    rpc_handler: Option<RpcHandlerFuture>,
    config: Option<ConfigFuture>,
    accounts: Option<AccountsFuture>,
    contacts: Option<ContactsFuture>,
    database: Option<DatabaseFuture>,
    notifier: Option<NotifierFuture>,
    streaming: Option<StreamingFuture>
}

impl <T>Builder<T> {

    pub fn new() -> Builder<T> {
        Builder {
            rpc: None,
            rpc_handler: None,
            config: None,
            accounts: None,
            contacts: None,
            database: None,
            notifier: None,
            streaming: None
        }
    }

    pub fn rpc(mut self, rpc: RpcFuture<T>) -> Builder<T> {
        self.rpc = Some(rpc);
        self
    }

    pub fn rpc_handler(mut self, handler: RpcHandlerFuture) -> Builder<T> {
        self.rpc_handler = Some(handler);
        self
    }

    pub fn config(mut self, config: ConfigFuture) -> Builder<T> {
        self.config = Some(config);
        self
    }

    pub fn accounts(mut self, accounts: AccountsFuture) -> Builder<T> {
        self.accounts = Some(accounts);
        self
    }

    pub fn contacts(mut self, contacts: ContactsFuture) -> Builder<T> {
        self.contacts = Some(contacts);
        self
    }

    pub fn database(mut self, database: DatabaseFuture) -> Builder<T> {
        self.database = Some(database);
        self
    }

    pub fn notifier(mut self, notifier: NotifierFuture) -> Builder<T> {
        self.notifier = Some(notifier);
        self
    }

    pub fn streaming(mut self, streaming: StreamingFuture) -> Builder<T> {
        self.streaming = Some(streaming);
        self
    }

    pub fn build(self) -> NirahResult<Server<T>> {
        let address_manager = AddressManager::new(5060);
        let sessions = HashMap::new();
        let config = self.config.unwrap_or(Box::new(InMemoryConfigProvider::new()));
        let accounts = self.accounts.unwrap_or(Box::new(InMemoryAccountsProvider::new()));
        let contacts = self.contacts.unwrap_or(Box::new(InMemoryContactsProvider::new()));
        let database = self.database.unwrap_or(Box::new(InMemoryDatabaseProvider::new()));
        let rpc_handler = self.rpc_handler.unwrap_or(Box::new(DefaultRpcHandler::new()));
        let notifier = self.notifier.unwrap_or(Box::new(NullNotifierProvider));
        #[cfg(not(feature = "gst"))]
        let streaming = self.streaming.unwrap_or(Box::new(NullStreamingProvider));
        #[cfg(feature = "gst")]
        let streaming = self.streaming.unwrap_or(Box::new(GStreamerProvider::new()?));
        if let Some(rpc) = self.rpc {
            Ok(Server {
                config, accounts, rpc_handler,
                rpc, address_manager, sessions,
                contacts, database, notifier,
                streaming
            })
        } else {
            Err(NirahError::MissingProvider("rpc"))
        }
    }
}
