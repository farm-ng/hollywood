use std::sync::Arc;

use self::{
    outbound_connection::{ActiveConnection, ConnectionConfig},
    request_connection::{
        ActiveRequestConnection, GenericRequestConnection, RequestConnectionConfig,
    },
};

use super::outbound::GenericConnection;

/// Infrastructure to connect an outbound channel of one actor to an inbound channel of another actor.
///
/// Note that the implementation is a bit over-engineered and can likely be simplified.
pub mod outbound_connection;

/// Infrastructure to connect an request channel of one actor to an inbound channel of another actor.
///
/// Note that the implementation is a bit over-engineered and can likely be simplified.
pub mod request_connection;

type ConnectionRegister<T> = Vec<Arc<dyn GenericConnection<T> + Send + Sync>>;

/// Connection 
pub enum ConnectionEnum<T> {
    /// Configuration of the connection
    Config(ConnectionConfig<T>),
    /// Active connection
    Active(ActiveConnection<T>),
}

type RequestConnectionRegister<T> = Option<Arc<dyn GenericRequestConnection<T> + Send + Sync>>;

pub(crate) enum RequestConnectionEnum<T> {
    Config(RequestConnectionConfig<T>),
    Active(ActiveRequestConnection<T>),
}
