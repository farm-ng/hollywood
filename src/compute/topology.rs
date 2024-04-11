use crate::introspect::flow_graph::FlowGraph;
use crate::prelude::*;
use petgraph::stable_graph::StableDiGraph;
use std::collections::BTreeSet;

// A node in a compute graph.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ActorNode {
    pub name: String,
    pub inbound: BTreeSet<String>,
    pub outbound: BTreeSet<String>,
}

/// Connection between two actors.
#[derive(Clone, Debug)]
pub struct Connection {
    /// name of the actor that owns the outbound channel
    pub from_actor: String,
    /// name of the outbound channel
    pub from: String,
    /// name of the actor that owns the inbound channel
    pub to: String,
    /// name of the inbound channel
    pub to_actor: String,
}

pub(crate) type HollywoodNodeIndex = petgraph::stable_graph::NodeIndex<u32>;

#[derive(Clone, Debug)]
pub(crate) struct UniqueNodeIdxNamePairs {
    node_idx_from_name: std::collections::HashMap<String, HollywoodNodeIndex>,
    name_from_node_idx: std::collections::HashMap<HollywoodNodeIndex, String>,
}

impl UniqueNodeIdxNamePairs {
    pub(crate) fn new() -> Self {
        UniqueNodeIdxNamePairs {
            node_idx_from_name: std::collections::HashMap::new(),
            name_from_node_idx: std::collections::HashMap::new(),
        }
    }

    pub(crate) fn try_insert(
        &mut self,
        name: String,
        node_idx: HollywoodNodeIndex,
    ) -> Result<(), String> {
        if self.node_idx_from_name.contains_key(&name) {
            return Err(format!("name {} already exists", name));
        }

        assert!(self
            .node_idx_from_name
            .insert(name.clone(), node_idx)
            .is_none());

        assert!(self
            .name_from_node_idx
            .insert(node_idx, name.clone())
            .is_none());

        Ok(())
    }

    pub(crate) fn get_node_idx(&self, name: &str) -> Option<HollywoodNodeIndex> {
        self.node_idx_from_name.get(name).cloned()
    }

    pub(crate) fn _get_name(&self, node_idx: HollywoodNodeIndex) -> Option<String> {
        self.name_from_node_idx.get(&node_idx).cloned()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Topology {
    pub(crate) graph: StableDiGraph<ActorNode, Connection, u32>,
    pub(crate) unique_idx_name_pairs: UniqueNodeIdxNamePairs,
}

impl Topology {
    pub(crate) fn new() -> Self {
        Topology {
            graph: StableDiGraph::new(),
            unique_idx_name_pairs: UniqueNodeIdxNamePairs::new(),
        }
    }

    pub(crate) fn add_new_unique_name(&mut self, name_hint: String) -> String {
        let mut count = 0;

        let node_idx = self.graph.add_node(ActorNode {
            name: name_hint.clone(),
            inbound: std::collections::BTreeSet::new(),
            outbound: std::collections::BTreeSet::new(),
        });

        let mut unique_name;

        loop {
            unique_name = format!("{}_{}", name_hint.clone(), count);
            if self
                .unique_idx_name_pairs
                .try_insert(unique_name.clone(), node_idx)
                .is_ok()
            {
                break;
            }
            count += 1;
        }
        self.graph[node_idx].name.clone_from(&unique_name.clone());

        unique_name
    }

    pub(crate) fn assert_unique_inbound_name(
        &mut self,
        unique_inbound_name: String,
        actor_name: &str,
    ) {
        let parent_idx = self.unique_idx_name_pairs.get_node_idx(actor_name).unwrap();
        let parent = self.graph.node_weight_mut(parent_idx).unwrap();

        if !parent.inbound.insert(unique_inbound_name.clone()) {
            panic!(
                "oh no, inbound name {} for {} already exists",
                unique_inbound_name, actor_name
            );
        }
    }

    pub(crate) fn assert_unique_outbound_name(
        &mut self,
        unique_outbound_name: String,
        actor_name: &str,
    ) {
        let parent_idx = self.unique_idx_name_pairs.get_node_idx(actor_name).unwrap();
        let parent = self.graph.node_weight_mut(parent_idx).unwrap();

        if !parent.outbound.insert(unique_outbound_name.clone()) {
            panic!(
                "oh no, outbound name {} for {} already exists",
                unique_outbound_name, actor_name
            );
        }
    }

    pub(crate) fn connect<
        T0: Clone + std::fmt::Debug + Sync + Send + 'static,
        T1: Clone + std::fmt::Debug + Sync + Send + 'static,
        M: IsInboundMessage,
    >(
        &mut self,
        outbound: &mut OutboundChannel<T0>,
        inbound: &mut InboundChannel<T1, M>,
    ) {
        let output_parent_idx = self
            .unique_idx_name_pairs
            .get_node_idx(&outbound.actor_name)
            .unwrap();
        let inbound_parent_idx = self
            .unique_idx_name_pairs
            .get_node_idx(&inbound.actor_name)
            .unwrap();
        assert_ne!(
            output_parent_idx, inbound_parent_idx,
            "oh no, outbound and inbound have same parent {} {}",
            &outbound.actor_name, &inbound.actor_name
        );
        self.graph.add_edge(
            output_parent_idx,
            inbound_parent_idx,
            Connection {
                from_actor: outbound.actor_name.clone(),
                from: outbound.name.clone(),
                to_actor: inbound.actor_name.clone(),
                to: inbound.name.clone(),
            },
        );
    }

    pub(crate) fn start_nodes(&self) -> Vec<ActorNode> {
        let start_nodes = self.graph.externals(petgraph::Direction::Incoming);
        start_nodes.map(|n| self.graph[n].clone()).collect()
    }

    pub(crate) fn pop_start_nodes(&mut self) -> Vec<ActorNode> {
        let start_nodes = self.start_nodes();
        for node in &start_nodes {
            let node_idx = self.unique_idx_name_pairs.get_node_idx(&node.name).unwrap();
            self.graph.remove_node(node_idx).unwrap();
        }
        start_nodes
    }

    pub(crate) fn analyze_graph_topology(&self) {
        let is_cyclic = petgraph::algo::is_cyclic_directed(&self.graph);
        assert!(!is_cyclic, "oh no, graph is cyclic: {}", is_cyclic);
    }

    pub fn print_flow_graph(&self) {
        let flow_graph = FlowGraph::new(self);
        flow_graph.print();
    }
}
