use std::io::{self, BufRead, Write};
use std::fs::File;

use std::collections::{HashMap, HashSet, VecDeque, BinaryHeap};
use std::iter::FromIterator;
use std::cmp::Ordering;

use petgraph::stable_graph::StableGraph;
use petgraph::graph::{DefaultIx, NodeIndex};
use petgraph::algo::{dijkstra};
use petgraph::dot::{Dot, Config};


#[derive(Debug, Clone)]
struct Node {
    x: usize,
    y: usize,
    c: String,
    index: NodeIndex<DefaultIx>,
}

impl Node {
    fn is_alphabetic(&self) -> bool {
        return self.c.chars().next().unwrap().is_alphabetic();
    }

    fn is_key(&self) -> bool {
        return self.is_alphabetic() && (self.c.to_lowercase() == self.c);
    }

    fn is_door(&self) -> bool {
        return self.is_alphabetic() && (self.c.to_uppercase() == self.c);
    }

    fn key_opens(&self, key: &String) -> bool {
        return self.c.to_lowercase() == *key;
    }
}

type DoorNodes = Vec<NodeIndex<DefaultIx>>;
type KeyNodes = Vec<NodeIndex<DefaultIx>>;

// return a vector of visible keys from current node
fn visible_doors_and_keys(node_index: NodeIndex<DefaultIx>, graph: &StableGraph<Node, usize>) -> (DoorNodes, KeyNodes) {
    let mut door_nodes: DoorNodes = Vec::new();
    let mut key_nodes: KeyNodes = Vec::new();

    let mut seen: HashSet<NodeIndex<DefaultIx>> = HashSet::new();

    let mut exploration: VecDeque<NodeIndex<DefaultIx>> = VecDeque::new();
    exploration.push_back(node_index);
    seen.insert(node_index);

    while !exploration.is_empty() {
        let current_node = graph.node_weight(exploration.pop_front().unwrap()).unwrap();

        if current_node.is_door() {
            door_nodes.push(current_node.index);
        } else if current_node.is_key() {
            key_nodes.push(current_node.index);
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
        visible_doors_and_keys(maze.node_index(6, 3), &maze.graph),
        (vec![], vec![maze.node_index(8, 3), maze.node_index(16, 1)]),
    );
}

struct Maze {
    graph: StableGraph<Node, usize>,
    nodes_map: HashMap<usize, HashMap<usize, NodeIndex<DefaultIx>>>,
    rows: usize,
    cols: usize,
}

impl Maze {
    fn node_index(&self, x: usize, y: usize) -> NodeIndex<DefaultIx> {
        return *self.nodes_map.get(&y).unwrap().get(&x).unwrap();
    }

    fn letters(&self, v: Vec<NodeIndex<DefaultIx>>) -> HashSet<String> {
        let mut output: HashSet<String> = HashSet::new();
        for i in v {
            let node: &Node = self.graph.node_weight(i).unwrap();
            if node.c.chars().next().unwrap().is_alphabetic() {
                output.insert(node.c.clone());
            }
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

    fn steps(&self, i: NodeIndex<DefaultIx>, j: NodeIndex<DefaultIx>) -> usize {
        let result: HashMap<NodeIndex<DefaultIx>, usize> =
            dijkstra(&self.graph, i, Some(j), |e| *e.weight());
        return *result.get(&j).unwrap();
    }

    fn grab(&mut self, i: NodeIndex<DefaultIx>) -> String {
        let node = self.graph.node_weight_mut(i).unwrap();

        let result = node.c.clone();
        node.c = ".".to_string();

        return result;
    }

    fn new() -> Maze {
        return Maze {
            graph: StableGraph::new(),
            nodes_map: HashMap::new(),
            rows: 0,
            cols: 0
        };
    }

    fn clone_from(&mut self, source: &Self) -> &mut Maze {
        self.graph = source.graph.clone();
        self.nodes_map = source.nodes_map.clone();
        self.rows = source.rows;
        self.cols = source.cols;
        return self;
    }

    fn print(&self) {
        for y in 1..(self.rows-1) {
            match self.nodes_map.get(&y) {
                Some(r) => {
                    for x in 1..(self.cols-1) {
                        match r.get(&x) {
                            Some(c) => print!("{}", self.graph.node_weight(self.node_index(x, y)).unwrap().c),
                            None => print!("#"),
                        }
                    }
                },
                None => {
                    for x in 1..(self.cols-1) {
                        print!("#");
                    }
                },
            }
            println!("");
        }
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

    assert_eq!(DoorNodes::new(), door_nodes);

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
        graph: StableGraph::new(),
        nodes_map: HashMap::new(),
        rows: rows,
        cols: cols,
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

    for ix in maze.graph.clone().node_indices() {
        let node = maze.graph.node_weight_mut(ix).unwrap();
        node.index = ix;

        if node.c == "." {
            node.c = " ".to_string();
        }

        maze.nodes_map.entry(node.y).or_insert(HashMap::new());
        maze.nodes_map.get_mut(&node.y).unwrap().entry(node.x).or_insert(ix);

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

    // simplify maze
    let mut still_simplifying: bool = true;
    while still_simplifying {
        still_simplifying = false;

        // A: "c" <-> B: " " <-> C: "D"
        //
        // remove nodes (ex. B) that:
        // 1. have only two incident edges
        // 2. are space nodes

        // replace those edges by gluing together both A and C
        // sum up edge weight

        for ix in maze.graph.clone().node_indices() {
            let node = maze.graph.node_weight_mut(ix).unwrap();

            if node.c != " " {
                continue;
            }

            let mut num_edges: usize = 0;
            let mut edge_weight: usize = 0;
            let mut other_nodes: Vec<NodeIndex<DefaultIx>> = Vec::new();

            // because there's no diagonal and this is a maze, there
            // won't be multiple edges to a neighbor so this works
            for jx in maze.graph.neighbors(ix) {
                num_edges += 1;
                other_nodes.push(jx);
            }

            for eix in maze.graph.edges(ix) {
                // edge weight will be the same in either direction
                edge_weight += *eix.weight();
            }

            if num_edges == 2 && (other_nodes[0] != other_nodes[1]) {
                maze.graph.remove_node(ix);

                maze.graph.add_edge(other_nodes[0], other_nodes[1], edge_weight);
                maze.graph.add_edge(other_nodes[1], other_nodes[0], edge_weight);

                println!("removed {:?}, connected from {:?} to {:?} weight {}",
                    ix, other_nodes[0], other_nodes[1], edge_weight);

                still_simplifying = true;
                break;
            }
        }
    }

    return maze;
}

fn collect_all(maze: &Maze) -> usize {
    return collect_all_given(maze).unwrap();
}

struct Search {
    maze: Maze,
    index: NodeIndex<DefaultIx>,
    path_length: usize,
    cost: i32,
    depth: usize,
}

impl Ord for Search {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cost.cmp(&other.cost)
    }
}

impl PartialOrd for Search {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Search {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for Search {}

fn collect_all_given(amaze: &Maze) -> Option<usize> {

    // make sure cost is negative - this makes this a min heap
    let mut search_space: BinaryHeap<Search> = BinaryHeap::new();

    {
        let mut new_maze: Maze = Maze::new();
        new_maze.clone_from(amaze);

        search_space.push(
            Search{
                maze: new_maze,
                index: amaze.find_start_index().unwrap(),
                path_length: 0,
                cost: 0,
                depth: 0,
            }
        );
    }
    let mut best_path: Option<usize> = None;

    while !search_space.is_empty() {
        // pop off best search so far
        let current_search = search_space.pop().unwrap();

        // if the best path is known, then ignore items that exceed it
        match best_path {
            Some(i) => {
                /*println!("length {} best {} current {} cost {} depth {}",
                    search_space.len(),
                    i,
                    current_search.path_length,
                    current_search.cost,
                    current_search.depth,
                );*/

                if current_search.path_length >= i {
                    //println!("!");
                    continue;
                }
            },
            None => {},
        }

        //current_search.maze.print();

        // what can I collect?
        let (_, key_nodes) = visible_doors_and_keys(current_search.index, &current_search.maze.graph);

        // BUT are there any keys left in the maze?
        let mut keys_left: i32 = 0;
        let mut doors_left: i32 = 0;

        for ix in current_search.maze.graph.node_indices() {
            let node = current_search.maze.graph.node_weight(ix).unwrap();
            if node.is_key() {
                keys_left += 1;
            }
            if node.is_door() {
                doors_left += 1;
            }
        }

        if keys_left == 0 {
            // update best_path
            match best_path {
                Some(i) => {
                    if current_search.path_length < i {
                        //println!("update {}", current_search.path_length);
                        best_path = Some(current_search.path_length);
                    }
                },
                None => {
                    //println!("new {}", current_search.path_length);
                    best_path = Some(current_search.path_length);
                },
            }

            continue;
        }

        for key in key_nodes {
            let key_node = current_search.maze.graph.node_weight(key).unwrap();

            let mut new_maze: Maze = Maze::new();
            new_maze.clone_from(&current_search.maze);

            new_maze.grab(key);

            // if I choose something, then open all doors it points to in the whole map
            for ix in new_maze.graph.clone().node_indices() {
                let node = new_maze.graph.node_weight(ix).unwrap();
                if node.is_door() && node.key_opens(&key_node.c) {
                    new_maze.grab(ix);
                }
            }

            //println!("pushing:");
            //new_maze.print();

            let new_path_length = current_search.path_length + current_search.maze.steps(current_search.index, key);

            // a heuristic function that estimates the cost of the cheapest path from n to the goal.
            // - it never overestimates the actual cost to get to the goal
            //
            // A* must examine all equally meritorious paths to find the optimal path.

            match best_path {
                Some(i) => {
                    if (new_path_length + keys_left as usize) < i {
                        // if a best path is known, optimize for finding a shorter one
                        search_space.push(
                            Search{
                                maze: new_maze,
                                index: key,
                                path_length: new_path_length,
                                cost: -(keys_left),
                                depth: current_search.depth + 1,
                            }
                        );
                    }
                },
                None => {
                    // if no best path is known, optimize for finding one
                    // this is used to prune other branches later
                    search_space.push(
                        Search{
                            maze: new_maze,
                            index: key,
                            path_length: new_path_length,
                            cost: -(keys_left),
                            depth: current_search.depth + 1,
                        }
                    );
                },
            }
        }
    }

    return best_path;
}

#[test]
fn test_test1() {
    let raw_map: Vec<Vec<char>> =
        vec!["#########".chars().collect(),
             "#b.A.@.a#".chars().collect(),
             "#########".chars().collect()];

    let maze = get_lines_as_maze(raw_map);

    assert_eq!(collect_all(&maze), 8);
}

#[test]
fn test_test2() {
    let raw_map: Vec<Vec<char>> =
        vec!["########################".chars().collect(),
             "#f.D.E.e.C.b.A.@.a.B.c.#".chars().collect(),
             "######################.#".chars().collect(),
             "#d.....................#".chars().collect(),
             "########################".chars().collect()];

    let maze = get_lines_as_maze(raw_map);

    assert_eq!(collect_all(&maze), 86);
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

    println!("{} steps", collect_all(&maze));
}
