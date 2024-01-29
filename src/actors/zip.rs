use std::cmp::Reverse;
use std::fmt::Debug;
use std::fmt::Display;

use hollywood_macros::zip_n;

use crate::compute::Context;
use crate::core::request::NullRequest;
use crate::core::Activate;
use crate::core::Actor;
use crate::core::DefaultRunner;
use crate::core::FromPropState;
use crate::core::InboundChannel;
use crate::core::InboundHub;
use crate::core::InboundMessage;
use crate::core::InboundMessageNew;
use crate::core::NullProp;
use crate::core::OnMessage;
use crate::core::OutboundChannel;
use crate::core::OutboundHub;

/// Type of the Xth inbound channel for the zip actor.
#[derive(Clone, Debug, Default)]
pub struct ZipPair<const N: usize, Key: PartialEq + Eq + PartialOrd + Ord, Value> {
    /// Key to associate message from different inbound channels with.
    pub key: Key,
    /// The value to be zipped.
    pub value: Value,
}

impl<const N: usize, Key: PartialEq + Eq + PartialOrd + Ord, T> PartialEq for ZipPair<N, Key, T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<const N: usize, Key: PartialEq + Eq + PartialOrd + Ord, T> Eq for ZipPair<N, Key, T> {}

impl<const N: usize, Key: PartialEq + Eq + PartialOrd + Ord, T> PartialOrd for ZipPair<N, Key, T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<const N: usize, Key: PartialEq + Eq + PartialOrd + Ord, T> Ord for ZipPair<N, Key, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}


zip_n!(2);
zip_n!(3);
zip_n!(4);
zip_n!(5);
zip_n!(6);
zip_n!(7);
zip_n!(8);
zip_n!(9);
zip_n!(10);
zip_n!(11);
zip_n!(12);
