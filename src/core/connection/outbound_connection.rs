use crate::core::connection::ConnectionRegister;
use crate::prelude::*;
use std::sync::Arc;

/// Connection configuration
pub struct ConnectionConfig<T> {
    /// List of connections
    pub connection_register: ConnectionRegister<T>,
    /// Launch pad for new connections
    pub maybe_register_launch_pad: Option<tokio::sync::oneshot::Sender<ConnectionRegister<T>>>,
    /// Landing pad for new connections
    pub maybe_register_landing_pad: Option<tokio::sync::oneshot::Receiver<ConnectionRegister<T>>>,
}

impl<T> Drop for ConnectionConfig<T> {
    fn drop(&mut self) {
        if let Some(connection_launch_pad) = self.maybe_register_launch_pad.take() {
            let connection_register = std::mem::take(&mut self.connection_register);
            let _ = connection_launch_pad.send(connection_register);
        } else {
            panic!("ConnectionConfig dropped when launch pad is is empty");
        }
    }
}

impl<T> ConnectionConfig<T> {
    /// Create connection configuration
    pub fn new() -> Self {
        let (connection_launch_pad, connection_landing_pad) = tokio::sync::oneshot::channel();
        Self {
            connection_register: vec![],
            maybe_register_launch_pad: Some(connection_launch_pad),
            maybe_register_landing_pad: Some(connection_landing_pad),
        }
    }
}

impl<T> Default for ConnectionConfig<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Active connection
pub struct ActiveConnection<T> {
    /// List of connections
    pub maybe_registers: Option<ConnectionRegister<T>>,
    /// Landing pad for new connections
    pub maybe_register_landing_pad: Option<tokio::sync::oneshot::Receiver<ConnectionRegister<T>>>,
}

impl<T: Clone + Send + Sync + std::fmt::Debug + 'static> Default for ConnectionEnum<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Send + Sync + std::fmt::Debug + 'static> ConnectionEnum<T> {
    /// new connection
    pub fn new() -> Self {
        Self::Config(ConnectionConfig::new())
    }

    /// push connection
    pub fn push(&mut self, connection: Arc<dyn IsGenericConnection<T> + Send + Sync>) {
        match self {
            Self::Config(config) => {
                config.connection_register.push(connection);
            }
            Self::Active(_) => {
                panic!("Cannot push to active connection");
            }
        }
    }

    pub(crate) fn send(&self, msg: T) {
        match self {
            Self::Config(_) => {
                panic!("Cannot send to config connection");
            }
            Self::Active(active) => {
                for i in active.maybe_registers.as_ref().unwrap().iter() {
                    i.send_impl(msg.clone());
                }
            }
        }
    }
}

impl<T> HasActivate for ConnectionEnum<T> {
    fn extract(&mut self) -> Self {
        match self {
            Self::Config(config) => Self::Active(ActiveConnection {
                maybe_registers: None,
                maybe_register_landing_pad: Some(config.maybe_register_landing_pad.take().unwrap()),
            }),
            Self::Active(_) => {
                panic!("Cannot extract active connection");
            }
        }
    }

    fn activate(&mut self) {
        match self {
            Self::Config(_) => {
                panic!("Cannot activate config connection");
            }
            Self::Active(active) => {
                let connection_register = active
                    .maybe_register_landing_pad
                    .take()
                    .unwrap()
                    .try_recv()
                    .unwrap();
                active.maybe_registers = Some(connection_register);
            }
        }
    }
}
