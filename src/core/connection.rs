use crate::core::connection::outbound_connection::ActiveConnection;
use crate::core::connection::outbound_connection::ConnectionConfig;
use crate::core::connection::request_connection::ActiveRequestConnection;
use crate::core::connection::request_connection::GenericRequestConnection;
use crate::core::connection::request_connection::RequestConnectionConfig;
use crate::prelude::*;
use std::sync::Arc;

/// Infrastructure to connect an outbound channel of one actor to an inbound channel of another actor.
///
/// Note that the implementation is a bit over-engineered and can likely be simplified.
pub mod outbound_connection;

/// Infrastructure to connect an request channel of one actor to an inbound channel of another actor.
///
/// Note that the implementation is a bit over-engineered and can likely be simplified.
pub mod request_connection;

type ConnectionRegister<T> = Vec<Arc<dyn IsGenericConnection<T> + Send + Sync>>;

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
