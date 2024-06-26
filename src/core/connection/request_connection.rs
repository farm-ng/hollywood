use crate::core::connection::RequestConnectionEnum;
use crate::core::connection::RequestConnectionRegister;
use crate::prelude::*;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::mpsc::error::SendError;
use tracing::warn;

pub(crate) trait GenericRequestConnection<T>: Send + Sync {
    fn send_impl(&self, msg: T);
}

#[derive(Debug, Clone)]
pub(crate) struct RequestConnection<T, M: IsInRequestMessage> {
    pub(crate) sender: tokio::sync::mpsc::UnboundedSender<M>,
    pub(crate) inbound_channel: String,
    pub(crate) phantom: PhantomData<T>,
}

impl<T: Send + Sync, M: IsInRequestMessageNew<T>> GenericRequestConnection<T>
    for RequestConnection<T, M>
{
    fn send_impl(&self, msg: T) {
        let msg = M::new(self.inbound_channel.clone(), msg);
        let c = self.sender.clone();
        let handler = tokio::spawn(async move {
            match c.send(msg) {
                Ok(_) => {}
                Err(SendError(e)) => {
                    warn!("Send request message error: {:?}", e);
                }
            }
        });
        std::mem::drop(handler);
    }
}

pub(crate) struct RequestConnectionConfig<T> {
    pub connection_register: RequestConnectionRegister<T>,
    pub maybe_register_launch_pad:
        Option<tokio::sync::oneshot::Sender<RequestConnectionRegister<T>>>,
    pub maybe_register_landing_pad:
        Option<tokio::sync::oneshot::Receiver<RequestConnectionRegister<T>>>,
}

impl<T> Drop for RequestConnectionConfig<T> {
    fn drop(&mut self) {
        if let Some(connection_launch_pad) = self.maybe_register_launch_pad.take() {
            let connection_register = std::mem::take(&mut self.connection_register);
            let _ = connection_launch_pad.send(connection_register);
        } else {
            panic!("ConnectionConfig dropped when launch pad is is empty");
        }
    }
}

impl<T> RequestConnectionConfig<T> {
    pub fn new() -> Self {
        let (connection_launch_pad, connection_landing_pad) = tokio::sync::oneshot::channel();
        Self {
            connection_register: None,
            maybe_register_launch_pad: Some(connection_launch_pad),
            maybe_register_landing_pad: Some(connection_landing_pad),
        }
    }
}

pub(crate) struct ActiveRequestConnection<T> {
    pub maybe_registers: Option<RequestConnectionRegister<T>>,
    pub maybe_register_landing_pad:
        Option<tokio::sync::oneshot::Receiver<RequestConnectionRegister<T>>>,
}

impl<T: Send + Sync + std::fmt::Debug + 'static> RequestConnectionEnum<T> {
    pub fn new() -> Self {
        Self::Config(RequestConnectionConfig::new())
    }

    pub fn push(&mut self, connection: Arc<dyn GenericRequestConnection<T> + Send + Sync>) {
        match self {
            Self::Config(config) => {
                assert!(config.connection_register.is_none());
                config.connection_register = Some(connection);
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
                let maybe_connection = active.maybe_registers.as_ref().unwrap();
                if maybe_connection.is_some() {
                    maybe_connection.as_ref().unwrap().send_impl(msg);
                }
            }
        }
    }
}

impl<T> HasActivate for RequestConnectionEnum<T> {
    fn extract(&mut self) -> Self {
        match self {
            Self::Config(config) => Self::Active(ActiveRequestConnection {
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
