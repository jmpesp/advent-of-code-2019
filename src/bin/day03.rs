use assert::equal;
use std::io::{self, BufRead};

#[derive(Debug)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

fn char_to_direction(i: char) -> Direction {
    match i {
        'U' => Direction::Up,
        'R' => Direction::Right,
        'D' => Direction::Down,
        'L' => Direction::Left,
        _ => {
            panic!();
        }
    }
}

type Length = i32;

#[derive(Debug)]
struct LineSegment {
    d: Direction,
    l: Length,
}

type Line = Vec<LineSegment>;

fn line_to_points(l: Line) -> Vec<Point> {
    let mut result: Vec<Point> = Vec::new();

    // all lines start at Point{0, 0}
    let mut p: Point = Point { x: 0, y: 0 };
    result.push(p.clone());

    for ls in l {
        p.add(ls);
        result.push(p.clone());
    }

    return result;
}

#[test]
fn test_line_to_points() {
    let input: Line = vec![
        LineSegment {
            d: Direction::Right,
            l: 10,
        },
        LineSegment {
            d: Direction::Down,
            l: 10,
        },
        LineSegment {
            d: Direction::Right,
            l: 10,
        },
        LineSegment {
            d: Direction::Up,
            l: 10,
        },
        LineSegment {
            d: Direction::Left,
            l: 20,
        },
    ];

    let expected: Vec<Point> = vec![
        Point { x: 0, y: 0 },
        Point { x: 10, y: 0 },
        Point { x: 10, y: 10 },
        Point { x: 20, y: 10 },
        Point { x: 20, y: 0 },
        Point { x: 0, y: 0 },
    ];

    equal(line_to_points(input), expected);
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn add(&mut self, ls: LineSegment) {
        match ls.d {
            Direction::Up => self.y = self.y - ls.l,
            Direction::Down => self.y = self.y + ls.l,

            Direction::Right => self.x = self.x + ls.l,
            Direction::Left => self.x = self.x - ls.l,
        }
    }
}

fn lines_from_input(inputs: Vec<String>) -> Vec<Line> {
    let mut result: Vec<Line> = Default::default();

    for input in inputs {
        let mut line: Line = Line::new();

        // R1002,D715,R356,D749,L255,U433,L558,D840,R933,U14,L285,U220,...
        for segment in input.split(',') {
            line.push(LineSegment {
                d: char_to_direction(segment.chars().next().unwrap()),
                l: segment[1..].parse::<i32>().unwrap(),
            });
        }

        result.push(line);
    }

    return result;
}

fn betweenf32(a: f32, b: f32, c: f32) -> bool {
    return a <= b && b <= c;
}

fn cramer_intersection(
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    x3: i32,
    y3: i32,
    x4: i32,
    y4: i32,
) -> Option<Point> {
    // s(x2-x1) - t(x4-x3) = x3-x1
    // s(y2-y1) - t(y4-y3) = y3-y1

    let a1 = x2 - x1;
    let b1 = -(x4 - x3);
    let c1 = x3 - x1;

    let a2 = y2 - y1;
    let b2 = -(y4 - y3);
    let c2 = y3 - y1;

    let det = a1 * b2 - b1 * a2;
    if det == 0 {
        return None;
    }

    let s = (c1 * b2 - b1 * c2) as f32 / det as f32;
    let t = (a1 * c2 - c1 * a2) as f32 / det as f32;

    if betweenf32(0.0, s, 1.0) && betweenf32(0.0, t, 1.0) {
        let x0 = x1 + (s * (x2 - x1) as f32) as i32;
        let y0 = y1 + (s * (y2 - y1) as f32) as i32;

        return Some(Point { x: x0, y: y0 });
    }

    return None;
}

fn find_intersections(l1: Line, l2: Line) -> Vec<Point> {
    let mut result: Vec<Point> = Default::default();

    let l1points = line_to_points(l1);
    let l2points = line_to_points(l2);

    // do not consider an intersection at origin
    // if not considering this, then first two segments can't intersect unless they overlap
    for i1 in 0..(l1points.len() - 1) {
        let p1a = l1points[i1 + 0];
        let p1b = l1points[i1 + 1];

        for i2 in 1..(l2points.len() - 1) {
            // TODO O(n^2), prune some tests?
            let p2a = l2points[i2 + 0];
            let p2b = l2points[i2 + 1];

            match cramer_intersection(p1a.x, p1a.y, p1b.x, p1b.y, p2a.x, p2a.y, p2b.x, p2b.y) {
                Some(p) => {
                    result.push(p);
                }
                None => {}
            }
        }
    }

    // extra case:
    let i1 = 1;
    let i2 = 0;

    let p1a = l1points[i1 + 0];
    let p1b = l1points[i1 + 1];

    let p2a = l2points[i2 + 0];
    let p2b = l2points[i2 + 1];

    match cramer_intersection(p1a.x, p1a.y, p1b.x, p1b.y, p2a.x, p2a.y, p2b.x, p2b.y) {
        Some(p) => {
            result.push(p);
        }
        None => {}
    }

    return result;
}

fn manhattan_distance(p: Point) -> usize {
    return p.x.abs() as usize + p.y.abs() as usize;
}

fn find_closest_intersection(l1: Line, l2: Line) -> usize {
    let mut result: Option<usize> = None;

    for intersection in find_intersections(l1, l2) {
        let md = manhattan_distance(intersection);
        match result {
            Some(i) => {
                if md < i {
                    result = Some(md);
                }
            }
            None => {
                result = Some(md);
            }
        }
    }

    return result.unwrap();
}

fn test_harness(sl1: String, sl2: String, expected_length: usize) {
    let input: Vec<String> = vec![sl1, sl2];
    let lines: Vec<Line> = lines_from_input(input);
    let mut lines_iter = lines.into_iter();
    let l1: Line = lines_iter.next().unwrap();
    let l2: Line = lines_iter.next().unwrap();

    assert_eq!(find_closest_intersection(l1, l2), expected_length);
}

#[test]
fn test1() {
    test_harness("R8,U5,L5,D3".to_string(), "U7,R6,D4,L4".to_string(), 6)
}

#[test]
fn test2() {
    test_harness(
        "R75,D30,R83,U83,L12,D49,R71,U7,L72".to_string(),
        "U62,R66,U55,R34,D71,R55,D58,R83".to_string(),
        159,
    );
}

#[test]
fn test3() {
    test_harness(
        "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51".to_string(),
        "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7".to_string(),
        135,
    );
}

fn main() {
    let reader = io::stdin();
    let input: Vec<String> = reader.lock().lines().map(|s| s.unwrap()).collect();

    let lines = lines_from_input(input);
    for line in &lines {
        println!("{:?}", line)
    }

    // find intersections
    let mut lines_iter = lines.into_iter();
    let l1: Line = lines_iter.next().unwrap();
    let l2: Line = lines_iter.next().unwrap();

    println!("{:?}", find_closest_intersection(l1, l2));
}
