use std::io::{self, BufRead, Write};
use std::fs::File;

use std::collections::{HashMap, HashSet, VecDeque};
use std::iter::FromIterator;

use petgraph::Graph;
use petgraph::graph::{DefaultIx, NodeIndex};
use petgraph::dot::{Dot, Config};

#[derive(Debug)]
struct Node {
    x: usize,
    y: usize,
    c: String,
    index: NodeIndex<DefaultIx>,
}

type DoorNodes = Vec<NodeIndex<DefaultIx>>;
type KeyNodes = Vec<NodeIndex<DefaultIx>>;

// return a vector of visible keys from current node
fn visible_doors_and_keys(node_index: NodeIndex<DefaultIx>, graph: &Graph<Node, usize>) -> (DoorNodes, KeyNodes) {
    let mut door_nodes: DoorNodes = Vec::new();
    let mut key_nodes: KeyNodes = Vec::new();

    let mut seen: HashSet<NodeIndex<DefaultIx>> = HashSet::new();

    let mut exploration: VecDeque<NodeIndex<DefaultIx>> = VecDeque::new();
    exploration.push_back(node_index);
    seen.insert(node_index);

    while !exploration.is_empty() {
        let current_node = graph.node_weight(exploration.pop_front().unwrap()).unwrap();

        if current_node.c.chars().next().unwrap().is_alphabetic() {
            if current_node.c == current_node.c.to_uppercase() {
                door_nodes.push(current_node.index);
            } else if current_node.c == current_node.c.to_lowercase() {
                key_nodes.push(current_node.index);
            }
        } else {
            for neighbour_index in graph.neighbors(current_node.index) {
                if !seen.contains(&neighbour_index) {
                    exploration.push_back(neighbour_index);
                    seen.insert(neighbour_index);
                }
            }
        }
    }

    return (door_nodes, key_nodes);
}

#[test]
fn test_visible_doors_and_keys() {
    let raw_map: Vec<Vec<char>> =
        vec!["########################".chars().collect(),
             "#...............b.C.D.f#".chars().collect(),
             "#.######################".chars().collect(),
             "#.....@.a.B.c.d.A.e.F.g#".chars().collect(),
             "########################".chars().collect()];

    let maze = get_lines_as_maze(raw_map);

    assert_eq!(
        visible_doors_and_keys(maze.node_index(1, 1), &maze.graph),
        (vec![], vec![maze.node_index(8, 3), maze.node_index(16, 1)]),
    );

    assert_eq!(
        visible_doors_and_keys(maze.node_index(9, 3), &maze.graph),
        (vec![maze.node_index(10, 3)], vec![maze.node_index(8, 3)]),
    );
}

struct Maze {
    graph: Graph<Node, usize>,
    nodes_map: HashMap<usize, HashMap<usize, NodeIndex<DefaultIx>>>,
}

impl Maze {
    fn node_index(&self, x: usize, y: usize) -> NodeIndex<DefaultIx> {
        return *self.nodes_map.get(&y).unwrap().get(&x).unwrap();
    }

    fn letters(&self, v: Vec<NodeIndex<DefaultIx>>) -> HashSet<String> {
        let mut output: HashSet<String> = HashSet::new();
        for i in v {
            output.insert(self.graph.node_weight(i).unwrap().c.clone());
        }
        return output;
    }

    fn find_start_index(&self) -> Option<NodeIndex<DefaultIx>> {
        for node_index in self.graph.node_indices() {
            let node = self.graph.node_weight(node_index).unwrap();
            if node.c == "@" {
                return Some(node.index);
            }
        }
        return None
    }
}

#[test]
fn test_letters() {
    let raw_map_2: Vec<Vec<char>> =
        vec!["#################".chars().collect(),
             "#i.G..c...e..H.p#".chars().collect(),
             "########.########".chars().collect(),
             "#j.A..b...f..D.o#".chars().collect(),
             "########@########".chars().collect(),
             "#k.E..a...g..B.n#".chars().collect(),
             "########.########".chars().collect(),
             "#l.F..d...h..C.m#".chars().collect(),
             "#################".chars().collect()];

    let maze_2 = get_lines_as_maze(raw_map_2);

    let (door_nodes, key_nodes) = visible_doors_and_keys(maze_2.node_index(8, 4), &maze_2.graph);

    let expected_key_nodes: HashSet<String> =
        HashSet::from_iter(
            vec!["a", "b", "c", "d", "e", "f", "g", "h"]
                .iter()
                .map(|s| s.to_string())
        );

    assert_eq!(maze_2.letters(key_nodes), expected_key_nodes);
}

#[test]
fn test_find_start_index() {
    let raw_map: Vec<Vec<char>> =
        vec!["#################".chars().collect(),
             "#i.G..c...e..H.p#".chars().collect(),
             "########.########".chars().collect(),
             "#j.A..b...f..D.o#".chars().collect(),
             "########@########".chars().collect(),
             "#k.E..a...g..B.n#".chars().collect(),
             "########.########".chars().collect(),
             "#l.F..d...h..C.m#".chars().collect(),
             "#################".chars().collect()];

    let maze = get_lines_as_maze(raw_map);

    assert_eq!(maze.find_start_index().unwrap(), maze.node_index(8, 4));
}

fn get_lines_as_maze(raw_map: Vec<Vec<char>>) -> Maze {
    println!("{:?}", raw_map);

    let rows = raw_map.len();
    let cols = raw_map[0].len();

    println!("{} {}", rows, cols);

    let mut maze: Maze = Maze{
        graph: Graph::new(),
        nodes_map: HashMap::new(),
    };

    // add nodes
    for y in 1..(rows-1) {
        for x in 1..(cols-1) {
            let point = raw_map[y][x];

            if point != '#' {
                maze.graph.add_node(Node{
                    x: x,
                    y: y,
                    c: point.to_string(),
                    index: NodeIndex::new(0),
                });
            }

            print!("{}", raw_map[y][x]);
        }
        print!("\n");
    }

    for node_index in maze.graph.node_indices() {
        let node = maze.graph.node_weight_mut(node_index).unwrap();
        node.index = node_index;

        maze.nodes_map.entry(node.y).or_insert(HashMap::new());
        maze.nodes_map.get_mut(&node.y).unwrap().entry(node.x).or_insert(node_index);

        println!("y {} x {} {:?}", node.y, node.x, node);
    }

    // add edges
    for y in 1..(rows-1) {
        for x in 1..(cols-1) {
            let point = raw_map[y][x];
            let north = raw_map[y-1][x];
            let south = raw_map[y+1][x];
            let west = raw_map[y][x-1];
            let east = raw_map[y][x+1];

            if point == '#' {
                continue
            }

            println!(" {} ", north);
            println!("{}{}{}", west, point, east);
            println!(" {} ", south);

            if north != '#' {
                println!("y {} x {} N", y, x);
                maze.graph.add_edge(
                    *maze.nodes_map.get(&y).unwrap().get(&x).unwrap(),
                    *maze.nodes_map.get(&(y-1)).unwrap().get(&x).unwrap(),
                    1
                );
            }
            if south != '#' {
                maze.graph.add_edge(
                    *maze.nodes_map.get(&y).unwrap().get(&x).unwrap(),
                    *maze.nodes_map.get(&(y+1)).unwrap().get(&x).unwrap(),
                    1
                );
            }
            if west != '#' {
                maze.graph.add_edge(
                    *maze.nodes_map.get(&y).unwrap().get(&x).unwrap(),
                    *maze.nodes_map.get(&y).unwrap().get(&(x-1)).unwrap(),
                    1
                );
            }
            if east != '#' {
                maze.graph.add_edge(
                    *maze.nodes_map.get(&y).unwrap().get(&x).unwrap(),
                    *maze.nodes_map.get(&y).unwrap().get(&(x+1)).unwrap(),
                    1
                );
            }

            println!("");
        }
    }

    return maze;
}

fn main() {
    let reader = io::stdin();
    let raw_map: Vec<Vec<char>> =
        reader.lock()
              .lines()
              .map(|s| s.unwrap().chars().collect())
              .collect();

    let maze = get_lines_as_maze(raw_map);

    let text = format!("{:?}", Dot::with_config(&maze.graph, &[Config::EdgeNoLabel]));
    println!("{}", text);

    let mut file = File::create("graph.dot").expect("failed to create graph.dot");
    file.write(&text.into_bytes()).expect("could not write into graph.dot");
}
