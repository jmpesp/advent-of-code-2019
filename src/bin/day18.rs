use std::io::{self, BufRead, Write};
use std::fs::File;
use std::fmt;

use std::process::exit;
use std::collections::HashMap;

use petgraph::Graph;
use petgraph::graph::{DefaultIx, NodeIndex};
use petgraph::dot::{Dot, Config};

#[derive(Debug)]
struct Node {
    x: usize,
    y: usize,
    c: char,
    index: NodeIndex<DefaultIx>,
}

fn main() {
    let reader = io::stdin();
    let raw_map: Vec<Vec<char>> =
        reader.lock()
              .lines()
              .map(|s| s.unwrap().chars().collect())
              .collect();

    println!("{:?}", raw_map);

    let rows = raw_map.len();
    let cols = raw_map[0].len();

    println!("{} {}", rows, cols);

    let mut maze: Graph<Node, usize> = Graph::new();
    let mut nodes_map: HashMap<usize, HashMap<usize, NodeIndex<DefaultIx>>> = HashMap::new();

    // add nodes
    for y in 1..(rows-1) {
        for x in 1..(cols-1) {
            let point = raw_map[y][x];

            if point != '#' {
                maze.add_node(Node{
                    x: x,
                    y: y,
                    c: point,
                    index: NodeIndex::new(0),
                });
            }

            print!("{}", raw_map[y][x]);
        }
        print!("\n");
    }

    for node_index in maze.node_indices() {
        let node = maze.node_weight_mut(node_index).unwrap();
        node.index = node_index;

        nodes_map.entry(node.y).or_insert(HashMap::new());
        nodes_map.get_mut(&node.y).unwrap().entry(node.x).or_insert(node_index);

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
                maze.add_edge(
                    *nodes_map.get(&y).unwrap().get(&x).unwrap(),
                    *nodes_map.get(&(y-1)).unwrap().get(&x).unwrap(),
                    1
                );
            }
            if south != '#' {
                maze.add_edge(
                    *nodes_map.get(&y).unwrap().get(&x).unwrap(),
                    *nodes_map.get(&(y+1)).unwrap().get(&x).unwrap(),
                    1
                );
            }
            if west != '#' {
                maze.add_edge(
                    *nodes_map.get(&y).unwrap().get(&x).unwrap(),
                    *nodes_map.get(&y).unwrap().get(&(x-1)).unwrap(),
                    1
                );
            }
            if east != '#' {
                maze.add_edge(
                    *nodes_map.get(&y).unwrap().get(&x).unwrap(),
                    *nodes_map.get(&y).unwrap().get(&(x+1)).unwrap(),
                    1
                );
            }

            println!("");
        }
    }

    let text = format!("{:?}", Dot::with_config(&maze, &[Config::EdgeNoLabel]));
    println!("{}", text);

    let mut file = File::create("graph.dot").expect("failed to create graph.dot");
    file.write(&text.into_bytes()).expect("could not write into graph.dot");
}
