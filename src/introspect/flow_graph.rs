use std::collections::{HashMap, VecDeque};

use drawille::{Canvas, PixelColor};
use grid::Grid;
use rand_distr::num_traits::{self, FromPrimitive, ToPrimitive};

use crate::compute::{ActorNode, Topology};

/// A super node is an actor with its inbound and outbound channels.
pub(crate) struct FlowSuperNode {
    pub inbound: Vec<String>,
    pub actor: String,
    pub outbound: Vec<String>,
}

impl FlowSuperNode {
    /// The width of the super node is the number of cells it occupies.
    pub fn width(&self) -> usize {
        self.inbound.len().max(self.outbound.len())
    }
}

/// A layer is a collection of super nodes.
pub(crate) struct FlowLayer {
    super_nodes: Vec<FlowSuperNode>,
}

impl FlowLayer {
    /// The width of the layer is the number of cells it occupies.
    pub fn width(&self) -> usize {
        self.super_nodes.iter().map(|n| n.width()).sum()
    }
}

/// A flow graph is an ordered collection of layers.
pub(crate) struct FlowGraph {
    pub layers: Vec<FlowLayer>,
    pub topology: Topology,
}

/// Given an input string, return a string of the exact length.
fn exact_length(s: &str, length: usize, left_b: char, right_b: char) -> String {
    if s.len() > length {
        let result = left_b.to_string() + &s[..length] + &right_b.to_string();
        assert_eq!(result.len(), length + 2);
        return result;
    }

    let extra_space = length - s.len();
    let extra_space_left = extra_space / 2;
    let extra_space_right = extra_space - extra_space_left;

    let mut result = s.to_string();

    for _ in 0..extra_space_left {
        result = " ".to_owned() + &result;
    }

    for _ in 0..extra_space_right {
        result.push(' ');
    }
    result = left_b.to_string() + &result + &right_b.to_string();
    assert_eq!(result.len(), length + 2);
    result
}

/// A primitive type
pub trait Primitive:
    num_traits::bounds::Bounded
    + Clone
    + std::fmt::Debug
    + std::fmt::Display
    + Copy
    + ToPrimitive
    + FromPrimitive
{
}

impl Primitive for f32 {}
impl Primitive for u32 {}
impl Primitive for usize {}

/// A (x,y) coordinate in the canvas, e.g. for plotting lines.
#[derive(Clone, Debug, Copy)]
struct PixelCoord<T> {
    x: T,
    y: T,
}

impl<T: Primitive> PixelCoord<T> {
    fn new(x: T, y: T) -> Self {
        PixelCoord { x, y }
    }
}

/// A (u,v) coordinate in the character grid, e.g. placing text.
#[derive(Clone, Debug, Copy)]
struct CharCoord {
    u: usize,
    v: usize,
}

impl CharCoord {
    const X_SCALE: f32 = 2.0;
    const Y_SCALE: f32 = 4.0;

    fn new(u: usize, v: usize) -> Self {
        CharCoord { u, v }
    }

    /// Convert a character coordinate to a canvas coordinate.
    fn to_canvas_coord<T: Primitive>(self) -> PixelCoord<T> {
        PixelCoord::new(
            T::from_f32(self.u.to_f32().unwrap() * Self::X_SCALE).unwrap(),
            T::from_f32(self.v.to_f32().unwrap() * Self::Y_SCALE).unwrap(),
        )
    }
}

/// A (col_id, row_id) coordinate in the cell grid. A cell has a height of 1 character and a width
/// of CELL_WIDTH characters.
#[derive(Clone, Debug, Copy)]
pub(crate) struct CellCoord {
    col_id: usize,
    row_id: usize,
}

impl CellCoord {
    const CELL_WIDTH: usize = FlowGraph::CELL_WIDTH;
    /// Create a new cell coordinate.
    pub fn new(col_id: usize, row_id: usize) -> Self {
        CellCoord { col_id, row_id }
    }

    fn to_canvas_coord<T: Primitive>(self) -> PixelCoord<T> {
        self.to_char_coord().to_canvas_coord()
    }

    fn to_char_coord(self) -> CharCoord {
        CharCoord::new(self.col_id * Self::CELL_WIDTH, self.row_id)
    }
}

struct LineCandidate {
    start: CellCoord,
    end: Option<CellCoord>,
}

impl FlowGraph {
    const TOKEN_WIDTH: usize = 16;
    const CELL_WIDTH: usize = Self::TOKEN_WIDTH + 2;

    pub fn new(topology: &Topology) -> Self {
        let mut flow_graph = FlowGraph {
            layers: Vec::new(),
            topology: topology.clone(),
        };

        let mut graph_clone: Topology = topology.clone();

        loop {
            let next_nodes = graph_clone.pop_start_nodes();
            if next_nodes.is_empty() {
                break;
            }
            flow_graph.add_layer(next_nodes);
        }

        flow_graph
    }

    pub fn add_layer(&mut self, nodes: Vec<ActorNode>) {
        let mut layer = FlowLayer {
            super_nodes: Vec::new(),
        };

        for node in nodes {
            let mut super_node = FlowSuperNode {
                inbound: Vec::new(),
                actor: node.name.clone(),
                outbound: Vec::new(),
            };
            for inbound in node.inbound {
                super_node.inbound.push(inbound);
            }
            for outbound in node.outbound {
                super_node.outbound.push(outbound);
            }
            layer.super_nodes.push(super_node);
        }
        self.layers.push(layer);
    }

    pub fn max_width(&self) -> usize {
        let mut max = 0;
        for layer in &self.layers {
            let width = layer.width();
            if width > max {
                max = width;
            }
        }
        max
    }

    fn calculate_grid(&self) -> (Grid<String>, HashMap<String, Vec<LineCandidate>>) {
        let mut grid: Grid<String> = Grid::new(0, 0);

        let mut line_candidates = HashMap::<String, Vec<LineCandidate>>::new();

        for layer in &self.layers {
            let mut row = vec![];
            let mut id = 0;

            for super_node in &layer.super_nodes {
                for _ in 0..(super_node.width() - super_node.inbound.len()) / 2 {
                    row.push(exact_length("", Self::TOKEN_WIDTH, ' ', ' '));
                    id += 1;
                }
                for i_inbound in 0..super_node.inbound.len() {
                    let inbound = &super_node.inbound[i_inbound];
                    let inbound_xchar = exact_length(
                        inbound,
                        Self::TOKEN_WIDTH,
                        if i_inbound == 0 { '|' } else { ' ' },
                        if i_inbound + 1 == super_node.inbound.len() {
                            '|'
                        } else {
                            ' '
                        },
                    );
                    row.push(inbound_xchar);

                    let cand = line_candidates.get_mut(&format!(
                        "{}/{}",
                        super_node.actor.clone(),
                        inbound.clone()
                    ));
                    if let Some(c) = cand {
                        for m in c {
                            m.end = Some(CellCoord::new(id, grid.rows() - 1));
                        }
                    }
                    id += 1;
                }
                for _ in id..super_node.width() {
                    row.push(exact_length("", Self::TOKEN_WIDTH, ' ', ' '));
                }
            }
            while row.len() < self.max_width() {
                row.push(exact_length("", Self::TOKEN_WIDTH, ' ', ' '));
            }

            grid.push_row(row);

            row = vec![];
            let mut id = 0;

            for super_node in &layer.super_nodes {
                for _ in 0..(super_node.width() - 1) / 2 {
                    row.push(exact_length("", Self::TOKEN_WIDTH, ' ', ' '));
                    id += 1;
                }
                let actor_xchar = exact_length(&super_node.actor, Self::TOKEN_WIDTH, '*', '*');
                id += 1;

                row.push(actor_xchar);
                for _ in id..super_node.width() {
                    row.push(exact_length("", Self::TOKEN_WIDTH, ' ', ' '));
                }
            }
            while row.len() < self.max_width() {
                row.push(exact_length("", Self::TOKEN_WIDTH, ' ', ' '));
            }
            grid.push_row(row);

            row = vec![];
            let mut id = 0;
            for super_node in &layer.super_nodes {
                for _ in 0..(super_node.width() - super_node.outbound.len()) / 2 {
                    row.push(exact_length("", Self::TOKEN_WIDTH, ' ', ' '));
                    id += 1;
                }
                for i_outbound in 0..super_node.outbound.len() {
                    let outbound = &super_node.outbound[i_outbound];

                    let outbound_xchar = exact_length(
                        outbound,
                        Self::TOKEN_WIDTH,
                        if i_outbound == 0 { '|' } else { ' ' },
                        if i_outbound + 1 == super_node.outbound.len() {
                            '|'
                        } else {
                            ' '
                        },
                    );
                    row.push(outbound_xchar);

                    let node_id = self
                        .topology
                        .unique_idx_name_pairs.get_node_idx(&super_node.actor).unwrap();
                     

                    for i in self
                        .topology
                        .graph
                        .edges_directed(node_id, petgraph::Direction::Outgoing)
                    {
                        if i.weight().from == *outbound {
                            let v = format!("{}/{}", i.weight().to_actor, i.weight().to);

                            line_candidates.entry(v).or_default().push(LineCandidate {
                                start: CellCoord::new(id, grid.rows() + 1),
                                end: None,
                            });
                        }
                    }

                    id += 1;
                }
                for _ in id..super_node.width() {
                    row.push(exact_length("", Self::TOKEN_WIDTH, ' ', ' '));
                }
            }
            while row.len() < self.max_width() {
                row.push(exact_length("", Self::TOKEN_WIDTH, ' ', ' '));
            }

            grid.push_row(row);

            for _i in 0..3 {
                row = vec![];
                while row.len() < self.max_width() {
                    row.push(exact_length("", Self::TOKEN_WIDTH, ' ', ' '));
                }
                grid.push_row(row);
            }
        }
        (grid, line_candidates)
    }

    pub fn canvas(&self) -> Canvas {
        let (grid, lines) = self.calculate_grid();

        let c = CellCoord::new(self.max_width(), grid.rows()).to_canvas_coord::<u32>();
        let mut canvas = Canvas::new(c.x, c.y);

        let mut v = VecDeque::new();

        for row_idx in 0..grid.rows() {
            for col_idx in 0..grid.cols() {
                let cell = grid.get(row_idx, col_idx).unwrap();

                let grid_coord = CellCoord::new(col_idx, row_idx);
                let char_coord = grid_coord.to_char_coord();
                let canvas_coord = char_coord.to_canvas_coord::<u32>();

                v.push_back(cell);
                canvas.text(
                    canvas_coord.x,
                    canvas_coord.y,
                    FlowGraph::CELL_WIDTH as u32 * 2,
                    v.back().unwrap().as_str(),
                );
            }
        }

        let color_arr = [
            PixelColor::Red,
            PixelColor::Green,
            PixelColor::Yellow,
            PixelColor::Blue,
            PixelColor::Magenta,
            PixelColor::Cyan,
        ];

        let mut color_idx = 0;

        for lines_per_input in lines.values() {
            for line in lines_per_input {
                let mut start_c = line.start.to_char_coord();
                start_c.u += Self::TOKEN_WIDTH / 2;
                let start = start_c.to_canvas_coord::<u32>();
                if line.end.is_some() {
                    let mut end_c = line.end.unwrap().to_char_coord();
                    end_c.u += Self::TOKEN_WIDTH / 2;
                    let end = end_c.to_canvas_coord::<u32>();

                    canvas.line_colored(start.x, start.y, end.x, end.y, color_arr[color_idx]);

                    color_idx = (color_idx + 1) % color_arr.len();
                }
            }
        }
        canvas
    }

    pub fn print(&self) {
        println!("{}", self.canvas().frame());
    }
}
