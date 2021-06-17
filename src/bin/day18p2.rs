use std::fs::File;
use std::io::{self, BufRead, Write};

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;

use petgraph::algo::dijkstra;
use petgraph::dot::Dot;
use petgraph::graph::{DefaultIx, NodeIndex};
use petgraph::stable_graph::StableGraph;

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
fn visible_doors_and_keys(
    node_index: NodeIndex<DefaultIx>,
    graph: &StableGraph<Node, usize>,
) -> (DoorNodes, KeyNodes) {
    let mut door_nodes: DoorNodes = Vec::new();
    let mut key_nodes: KeyNodes = Vec::new();

    let mut seen: HashSet<NodeIndex<DefaultIx>> = HashSet::new();

    let mut exploration: VecDeque<NodeIndex<DefaultIx>> = VecDeque::new();
    exploration.push_back(node_index);
    seen.insert(node_index);

    {
        // terminate if a non-legal character is found
        let node = graph.node_weight(node_index).unwrap();

        if node.c == " ".to_string() || node.c == "@".to_string() || node.c == ".".to_string() {
        } else {
            println!("{:?}", node.c);
            assert_eq!(true, false);
        }
    }

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
    let raw_map: Vec<Vec<char>> = vec![
        "########################".chars().collect(),
        "#...............b.C.D.f#".chars().collect(),
        "#.######################".chars().collect(),
        "#.....@.a.B.c.d.A.e.F.g#".chars().collect(),
        "########################".chars().collect(),
    ];

    let maze = get_lines_as_maze(raw_map);

    assert_eq!(
        visible_doors_and_keys(maze.node_index("@".to_string()).unwrap(), &maze.graph),
        (
            vec![],
            vec![
                maze.node_index("a".to_string()).unwrap(),
                maze.node_index("b".to_string()).unwrap()
            ]
        ),
    );
}

struct Maze {
    graph: StableGraph<Node, usize>,
}

impl Maze {
    fn node_index(&self, c: String) -> Option<NodeIndex<DefaultIx>> {
        for node_index in self.graph.node_indices() {
            let node = self.graph.node_weight(node_index).unwrap();
            if node.c == c {
                return Some(node.index);
            }
        }
        return None;
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

    fn find_start_index(&self) -> NodeIndex<DefaultIx> {
        return self.node_index("@".to_string()).unwrap();
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
        };
    }

    fn clone_from(&mut self, source: &Self) -> &mut Maze {
        self.graph = source.graph.clone();
        return self;
    }
}

#[test]
fn test_letters() {
    let raw_map_2: Vec<Vec<char>> = vec![
        "#################".chars().collect(),
        "#i.G..c...e..H.p#".chars().collect(),
        "########.########".chars().collect(),
        "#j.A..b...f..D.o#".chars().collect(),
        "########@########".chars().collect(),
        "#k.E..a...g..B.n#".chars().collect(),
        "########.########".chars().collect(),
        "#l.F..d...h..C.m#".chars().collect(),
        "#################".chars().collect(),
    ];

    let maze_2 = get_lines_as_maze(raw_map_2);

    let (door_nodes, key_nodes) =
        visible_doors_and_keys(maze_2.node_index("@".to_string()).unwrap(), &maze_2.graph);

    assert_eq!(DoorNodes::new(), door_nodes);

    let expected_key_nodes: HashSet<String> = HashSet::from_iter(
        vec!["a", "b", "c", "d", "e", "f", "g", "h"]
            .iter()
            .map(|s| s.to_string()),
    );

    assert_eq!(maze_2.letters(key_nodes), expected_key_nodes);
}

fn get_lines_as_maze(raw_map: Vec<Vec<char>>) -> Maze {
    println!("{:?}", raw_map);

    let rows = raw_map.len();
    let cols = raw_map[0].len();
    let mut nodes_map: HashMap<usize, HashMap<usize, NodeIndex<DefaultIx>>> = HashMap::new();

    println!("{} {}", rows, cols);

    let mut maze: Maze = Maze {
        graph: StableGraph::new(),
    };

    // add nodes
    for y in 1..(rows - 1) {
        for x in 1..(cols - 1) {
            let point = raw_map[y][x];

            if point != '#' {
                maze.graph.add_node(Node {
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

        nodes_map.entry(node.y).or_insert(HashMap::new());
        nodes_map
            .get_mut(&node.y)
            .unwrap()
            .entry(node.x)
            .or_insert(ix);

        println!("y {} x {} {:?}", node.y, node.x, node);
    }

    // add edges
    for y in 1..(rows - 1) {
        for x in 1..(cols - 1) {
            let point = raw_map[y][x];
            let north = raw_map[y - 1][x];
            let south = raw_map[y + 1][x];
            let west = raw_map[y][x - 1];
            let east = raw_map[y][x + 1];

            if point == '#' {
                continue;
            }

            println!(" {} ", north);
            println!("{}{}{}", west, point, east);
            println!(" {} ", south);

            if north != '#' {
                println!("y {} x {} N", y, x);
                maze.graph.add_edge(
                    *nodes_map.get(&y).unwrap().get(&x).unwrap(),
                    *nodes_map.get(&(y - 1)).unwrap().get(&x).unwrap(),
                    1,
                );
            }
            if south != '#' {
                maze.graph.add_edge(
                    *nodes_map.get(&y).unwrap().get(&x).unwrap(),
                    *nodes_map.get(&(y + 1)).unwrap().get(&x).unwrap(),
                    1,
                );
            }
            if west != '#' {
                maze.graph.add_edge(
                    *nodes_map.get(&y).unwrap().get(&x).unwrap(),
                    *nodes_map.get(&y).unwrap().get(&(x - 1)).unwrap(),
                    1,
                );
            }
            if east != '#' {
                maze.graph.add_edge(
                    *nodes_map.get(&y).unwrap().get(&x).unwrap(),
                    *nodes_map.get(&y).unwrap().get(&(x + 1)).unwrap(),
                    1,
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

            // only prune spaces!
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

                maze.graph
                    .add_edge(other_nodes[0], other_nodes[1], edge_weight);
                maze.graph
                    .add_edge(other_nodes[1], other_nodes[0], edge_weight);

                println!(
                    "removed {:?}, connected from {:?} to {:?} weight {}",
                    ix, other_nodes[0], other_nodes[1], edge_weight
                );

                still_simplifying = true;
                break;
            }

            if num_edges == 1 {
                // extra: prune leaf nodes that are not special
                maze.graph.remove_node(ix);

                println!("removed {:?}, was non-special leaf", ix);

                still_simplifying = true;
                break;
            }
        }
    }

    return maze;
}

struct Search {
    maze: Maze,
    index: NodeIndex<DefaultIx>,
    path_length: usize,
    cost: i32,
    depth: usize,
    keys: HashSet<String>,
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

struct SearchState {
    index: NodeIndex<DefaultIx>,
    keys: HashSet<String>,
    path_length: usize,
}
impl Hash for SearchState {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        // do not hash path_length
        state.write_usize(self.index.index());
        for key in self.keys.clone() {
            key.hash(state);
        }
        state.finish();
    }
}
impl PartialEq for SearchState {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.keys == other.keys
    }
}
impl Eq for SearchState {}

fn collect_all(maze: &Maze) -> usize {
    return collect_all_given(maze).unwrap();
}

fn steps_to_farthest_key(node_index: NodeIndex<DefaultIx>, maze: &Maze) -> Option<usize> {
    let mut steps: Option<usize> = None;

    for ix in maze.graph.node_indices() {
        let node = maze.graph.node_weight(ix).unwrap();
        if node.is_key() {
            let key_steps = maze.steps(node_index, ix);

            match steps {
                Some(i) => {
                    if key_steps > i {
                        steps = Some(key_steps);
                    }
                }
                None => {
                    steps = Some(key_steps);
                }
            }
        }
    }

    return steps;
}

fn collect_all_given(amaze: &Maze) -> Option<usize> {
    // make sure cost is negative - this makes this a min heap
    let mut search_space: BinaryHeap<Search> = BinaryHeap::new();

    {
        let mut new_maze: Maze = Maze::new();
        new_maze.clone_from(amaze);

        search_space.push(Search {
            maze: new_maze,
            index: amaze.find_start_index(),
            path_length: 0,
            cost: 0,
            depth: 0,
            keys: ["".to_string()].iter().cloned().collect(),
        });
    }

    // best total path
    let mut best_path: Option<usize> = None;

    // closed set: if a node has already been examined, then don't re-examine, unless its
    // cost can be lowered
    let mut closed_set: HashSet<SearchState> = HashSet::new();

    while !search_space.is_empty() {
        // pop off best search so far
        let current_search = search_space.pop().unwrap();

        let reached: SearchState = SearchState {
            index: current_search.index,
            keys: current_search.keys.clone(),
            path_length: current_search.path_length,
        };
        if closed_set.contains(&reached) {
            // is our search better?
            {
                let already_reached: &SearchState = closed_set.get(&reached).unwrap();
                if already_reached.path_length <= reached.path_length {
                    continue;
                }
            }
            // if so, search
            // this works because already_reached and reached hash to the same thing
            closed_set.remove(&reached);
            closed_set.insert(reached);
        } else {
            // if new, search
            closed_set.insert(reached);
        }

        // what can I collect?
        let (_, key_nodes) =
            visible_doors_and_keys(current_search.index, &current_search.maze.graph);

        // BUT are there any keys left in the maze?
        let mut keys_left: i32 = 0;

        for ix in current_search.maze.graph.node_indices() {
            let node = current_search.maze.graph.node_weight(ix).unwrap();
            if node.is_key() {
                keys_left += 1;
            }
        }

        if keys_left == 0 {
            // update best_path
            match best_path {
                Some(i) => {
                    if current_search.path_length < i {
                        println!("update {}", current_search.path_length);
                        best_path = Some(current_search.path_length);
                    }
                }
                None => {
                    println!("new {}", current_search.path_length);
                    best_path = Some(current_search.path_length);
                }
            }

            continue;
        }

        match best_path {
            Some(i) => {
                // if the best path is known, then ignore items that are not better
                if current_search.path_length >= i {
                    continue;
                }
            }
            None => {}
        }

        /*
        println!("length {} current {} cost {} depth {} keys got {} keys left {}",
            search_space.len(),
            current_search.path_length,
            current_search.cost,
            current_search.depth,
            current_search.keys,
            keys_left,
        );
        */

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

            let new_path_length =
                current_search.path_length + current_search.maze.steps(current_search.index, key);

            // a heuristic function that estimates the cost of the cheapest path from n to the goal.
            // - it never overestimates the actual cost to get to the goal
            //
            // A* must examine all equally meritorious paths to find the optimal path.
            let farthest_key: Option<usize> = steps_to_farthest_key(key, &new_maze);

            match farthest_key {
                // there is some key still to get
                Some(_) => {
                    let cost: i32;

                    match best_path {
                        Some(i) => {
                            // the best possible path must be better to count (hence >=)
                            // note there might be a key along the way
                            if (new_path_length as i32 + farthest_key.unwrap() as i32) >= i as i32 {
                                continue;
                            }

                            // bfs to explore search space
                            //cost = farthest_key.unwrap() as i32;

                            // dfs
                            cost = keys_left;
                        }
                        None => {
                            // if no best path exists, dfs to find one
                            cost = keys_left;
                        }
                    }

                    let mut new_keys: HashSet<String> = current_search.keys.clone();
                    new_keys.insert(key_node.c.clone());

                    search_space.push(Search {
                        maze: new_maze,
                        index: key,
                        path_length: new_path_length,
                        cost: -(cost),
                        depth: current_search.depth + 1,
                        keys: new_keys,
                    });
                }
                // there is no other key, so the cost after this is zero
                None => {
                    let mut new_keys: HashSet<String> = current_search.keys.clone();
                    new_keys.insert(key_node.c.clone());

                    search_space.push(Search {
                        maze: new_maze,
                        index: key,
                        path_length: new_path_length,
                        cost: 0,
                        depth: current_search.depth + 1,
                        keys: new_keys,
                    });
                }
            }
        }
    }

    return best_path;
}

#[test]
fn test_test1() {
    let raw_map: Vec<Vec<char>> = vec![
        "#########".chars().collect(),
        "#b.A.@.a#".chars().collect(),
        "#########".chars().collect(),
    ];

    let maze = get_lines_as_maze(raw_map);

    assert_eq!(collect_all(&maze), 8);
}

#[test]
fn test_test2() {
    let raw_map: Vec<Vec<char>> = vec![
        "########################".chars().collect(),
        "#f.D.E.e.C.b.A.@.a.B.c.#".chars().collect(),
        "######################.#".chars().collect(),
        "#d.....................#".chars().collect(),
        "########################".chars().collect(),
    ];

    let maze = get_lines_as_maze(raw_map);

    assert_eq!(collect_all(&maze), 86);
}

#[test]
fn test_example1() {
    let raw_map: Vec<Vec<char>> = vec![
        "########################".chars().collect(),
        "#...............b.C.D.f#".chars().collect(),
        "#.######################".chars().collect(),
        "#.....@.a.B.c.d.A.e.F.g#".chars().collect(),
        "########################".chars().collect(),
    ];

    let maze = get_lines_as_maze(raw_map);

    assert_eq!(collect_all(&maze), 132);
}

#[test]
fn test_example3() {
    let raw_map: Vec<Vec<char>> = vec![
        "########################".chars().collect(),
        "#@..............ac.GI.b#".chars().collect(),
        "###d#e#f################".chars().collect(),
        "###A#B#C################".chars().collect(),
        "###g#h#i################".chars().collect(),
        "########################".chars().collect(),
    ];

    let maze = get_lines_as_maze(raw_map);

    assert_eq!(collect_all(&maze), 81);
}

fn main() {
    let reader = io::stdin();
    let raw_map: Vec<Vec<char>> = reader
        .lock()
        .lines()
        .map(|s| s.unwrap().chars().collect())
        .collect();

    let maze = get_lines_as_maze(raw_map);

    let text = format!("{:?}", Dot::with_config(&maze.graph, &[]));
    println!("{}", text);

    let mut file = File::create("graph.dot").expect("failed to create graph.dot");
    file.write(&text.into_bytes())
        .expect("could not write into graph.dot");

    println!("{} steps", collect_all(&maze));
}
