use std::collections::BTreeSet;

use petgraph::{algo::toposort, stable_graph::StableDiGraph};
use petgraph::visit::EdgeRef;

// A node in a compute graph.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct ActorNode {
    pub name: String,
    pub inbound: BTreeSet<String>,
    pub outbound: BTreeSet<String>,
}

// Connection between two actors.
struct Connection {
    pub from: String,
    pub to: String,
}

pub(crate) type HollywoodNodeIndex = petgraph::stable_graph::NodeIndex<u32>;

pub(crate) struct Topology {
    graph: StableDiGraph<ActorNode, Connection, u32>,
    node_idx_from_name: std::collections::HashMap<String, HollywoodNodeIndex>,
}

impl Topology {
    pub(crate) fn new() -> Self {
        Topology {
            graph: StableDiGraph::new(),
            node_idx_from_name: std::collections::HashMap::new(),
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
                .node_idx_from_name
                .insert(unique_name.clone(), node_idx)
                .is_none()
            {
                break;
            }
            count += 1;
        }

        self.graph[node_idx].name = unique_name.clone();

        unique_name
    }

    pub(crate) fn assert_unique_inbound_name(
        &mut self,
        unique_inbound_name: String,
        actor_name: &str,
    ) {
        let parent_idx = self.node_idx_from_name.get(actor_name).unwrap();
        let parent = self.graph.node_weight_mut(*parent_idx).unwrap();

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
        let parent_idx = self.node_idx_from_name.get(actor_name).unwrap();
        let parent = self.graph.node_weight_mut(*parent_idx).unwrap();

        if !parent.outbound.insert(unique_outbound_name.clone()) {
            panic!(
                "oh no, outbound name {} for {} already exists",
                unique_outbound_name, actor_name
            );
        }
    }

    pub(crate) fn connect(
        &mut self,
        actor_of_outbound_channel: &str,
        actor_of_inbound_channel: &str,
    ) {
        let output_parent_idx = self
            .node_idx_from_name
            .get(actor_of_outbound_channel)
            .unwrap();
        let inbound_parent_idx = self
            .node_idx_from_name
            .get(actor_of_inbound_channel)
            .unwrap();
        assert_ne!(
            output_parent_idx, inbound_parent_idx,
            "oh no, outbound and inbound have same parent {} {}",
            actor_of_outbound_channel, actor_of_inbound_channel
        );
        self.graph.add_edge(
            *output_parent_idx,
            *inbound_parent_idx,
            Connection {
                from: actor_of_inbound_channel.to_owned(),
                to: actor_of_inbound_channel.to_owned(),
            },
        );
    }

    fn get_node_name(&self, node_idx: HollywoodNodeIndex) -> String {
        self.graph.node_weight(node_idx).unwrap().name.clone()
    }

    pub(crate) fn analyze_graph_topology(&self) {
        let is_cyclic = petgraph::algo::is_cyclic_directed(&self.graph);
        assert!(!is_cyclic, "oh no, graph is cyclic: {}", is_cyclic);

        let start_nodes = self.graph.externals(petgraph::Direction::Incoming);
        print!("start nodes: ");
        for node in start_nodes.clone() {
            print!("{}, ", self.get_node_name(node));
        }
        println!(" ");
        let end_nodes = self.graph.externals(petgraph::Direction::Outgoing);
        print!("end nodes: ");
        for node in end_nodes {
            print!("{}, ", self.get_node_name(node));
        }
        println!(" ");

        let l: Vec<petgraph::stable_graph::NodeIndex> = toposort(&self.graph, None).unwrap();
        for node in l {
            let name = self.get_node_name(node);

            let mut in_neighbors: Vec<petgraph::stable_graph::NodeIndex> = self
                .graph
                .neighbors_directed(node, petgraph::Direction::Incoming)
                .collect();
            let inbound = in_neighbors.len();
            in_neighbors.sort();
            in_neighbors.dedup();
            let deduped_inbounds = in_neighbors.len();
            let mut out_neighbors: Vec<petgraph::stable_graph::NodeIndex> = self
                .graph
                .neighbors_directed(node, petgraph::Direction::Outgoing)
                .collect();
            let out = out_neighbors.len();
            out_neighbors.sort();
            out_neighbors.dedup();
            let deduped_outputs = out_neighbors.len();

            println!(
                "[#{} in from {} actors]: {} [#{} out to {} actors]",
                inbound, deduped_inbounds, name, out, deduped_outputs,
            );

            for neighbor in out_neighbors {
                let neighbor_name = self.graph.node_weight(neighbor).unwrap().name.clone();
                println!("=> {}, ", neighbor_name);

                for e in self.graph.edges_connecting(node, neighbor) {
                    println!(" {} ==> {}", e.weight().from, e.weight().to);
                }
                println!(" ");
            }
            println!(" ");
        }
        println!();

        let mut row: Vec<HollywoodNodeIndex> = vec![];

        for node in start_nodes {
            row.push(node);
        }
        let mut count = 0;
        loop {
            let mut next_row: Vec<HollywoodNodeIndex> = vec![];
            println!("row {}: ", count);
            for node in row {
                let mut in_neighbors: Vec<petgraph::stable_graph::NodeIndex> = self
                    .graph
                    .neighbors_directed(node, petgraph::Direction::Incoming)
                    .collect();
                in_neighbors.sort();
                in_neighbors.dedup();
                for neighbor in in_neighbors {
                    for e in self.graph.edges_connecting(neighbor, node) {
                        print!("{} ==> {} | ", e.id().index(), e.weight().to);
                    }
                }
                println!();
                println!("{} | ", self.get_node_name(node));
                let mut out_neighbors: Vec<petgraph::stable_graph::NodeIndex> = self
                    .graph
                    .neighbors_directed(node, petgraph::Direction::Outgoing)
                    .collect();
                out_neighbors.sort();
                out_neighbors.dedup();
                for neighbor in out_neighbors {
                    next_row.push(neighbor);
                    for e in self.graph.edges_connecting(node, neighbor) {
                        print!(" {} ==> {} | ", e.weight().from, e.id().index());
                    }
                }
            }
            if next_row.is_empty() {
                break;
            }
            row = next_row;
            println!(" ");
            count += 1;
        }
        println!();
        println!();
    }
}
