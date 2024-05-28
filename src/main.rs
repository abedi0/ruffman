use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    env::args,
    fs::File,
    io::{self, Error, Read, Write},
};

use bitvec::prelude::*;

trait HasWeight {
    fn weight(&self) -> u32;
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
struct LeafNode {
    weight: u32,
    symb: char,
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
struct InternalNode {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
    weight: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
enum Node {
    Leaf(LeafNode),
    Internal(InternalNode),
}

impl HasWeight for Node {
    fn weight(&self) -> u32 {
        match self {
            Node::Leaf(leaf) => leaf.weight,
            Node::Internal(internal) => internal.weight,
        }
    }
}

#[derive(Clone, Debug)]
struct NodeBytes {
    input: String,
    node: Node,
    bytes: Vec<u8>,
}

impl From<Node> for NodeBytes {
    fn from(value: Node) -> Self {
        Self {
            input: String::new(),
            node: value,
            bytes: vec![],
        }
    }
}

impl From<Vec<u8>> for NodeBytes {
    fn from(value: Vec<u8>) -> Self {
        let mut bytes = Self {
            input: String::new(),
            node: Node::Internal(InternalNode {
                left: None,
                right: None,
                weight: 0,
            }),
            bytes: value,
        };

        bytes.into_node();

        bytes
    }
}

impl NodeBytes {
    fn as_bytes(&mut self) {
        self.as_bytes_rec(Box::new(self.node.clone()));
    }

    // [(symb, weight), second_node]
    fn as_bytes_rec(&mut self, node: Box<Node>) {
        match *node {
            Node::Internal(internal) => {
                if let Some(left_node) = internal.left {
                    self.bytes.push(0);
                    self.as_bytes_rec(left_node);
                }

                if let Some(right_node) = internal.right {
                    self.bytes.push(0);
                    self.as_bytes_rec(right_node);
                }
            }
            Node::Leaf(leaf) => {
                self.bytes.push(1);
                self.bytes.push(leaf.symb as u8);
            }
        }
    }

    fn into_node(&mut self) {
        let mut nodes = BinaryHeap::new();
        let mut bytes_iter = self.bytes.clone().into_iter();
        while let Some(val) = bytes_iter.next() {
            if val == 1 {
                nodes.push(Reverse(Node::Leaf(LeafNode {
                    weight: 0,
                    symb: bytes_iter.next().unwrap() as char,
                })));
            }
        }
        while nodes.len() > 1 {
            let node0 = nodes.pop().unwrap();
            let n0w = node0.0.weight();
            let node1 = nodes.pop().unwrap();
            let n1w = node1.0.weight();
            let new_node = InternalNode {
                left: Some(Box::new(node0.0)),
                right: Some(Box::new(node1.0)),

                weight: n0w + n1w,
            };

            nodes.push(Reverse(Node::Internal(new_node)));
        }

        self.node = nodes.pop().unwrap().0
    }

    fn gen_input(&mut self) {
        fn get_input_req(node: Node, result: &mut String) {
            match node {
                Node::Leaf(leaf) => {
                    for _i in 0..leaf.weight {
                        result.push(leaf.symb);
                    }
                }

                Node::Internal(internal) => {
                    if let Some(left_node) = internal.left {
                        get_input_req(*left_node, result);
                    }

                    if let Some(right_node) = internal.right {
                        get_input_req(*right_node, result);
                    }
                }
            }
        }

        get_input_req(self.node.clone(), &mut self.input);
    }
}

fn calc_huff(n: Vec<(char, u32)>) -> Node {
    let mut set = BinaryHeap::new();

    for i in n {
        let new_node = LeafNode {
            symb: i.0,

            weight: i.1,
        };

        set.push(Reverse(Node::Leaf(new_node)));
    }

    while set.len() > 1 {
        let node0 = set.pop().unwrap();
        let n0w = node0.0.weight();
        let node1 = set.pop().unwrap();
        let n1w = node1.0.weight();

        let new_node = InternalNode {
            left: Some(Box::new(node0.0)),
            right: Some(Box::new(node1.0)),

            weight: n0w + n1w,
        };

        set.push(Reverse(Node::Internal(new_node)));
    }

    return set.pop().unwrap().0;
}

fn calc_freq(input: String) -> Vec<(char, u32)> {
    let mut freqs: Vec<(char, u32)> = Vec::new();

    for char in input.chars() {
        if let Some(pos) = freqs
            .clone()
            .into_iter()
            .position(|(c, _v)| c.clone() == char)
        {
            freqs[pos].1 += 1;
        } else {
            freqs.push((char, 1));
        }
    }

    freqs
}

#[derive(Clone)]
struct Huffman {
    input: String,
    char_codes: HashMap<char, Vec<u8>>,
    tree: Node,
}

impl From<Node> for Huffman {
    fn from(value: Node) -> Self {
        Self {
            tree: value,
            input: String::new(),
            char_codes: HashMap::new(),
        }
    }
}

impl Huffman {
    pub fn from_input(input: String) -> Self {
        let tree = calc_huff(calc_freq(input.clone()));

        Self {
            tree,
            input,
            char_codes: HashMap::new(),
        }
    }

    fn compress(&mut self) {
        self.huff_compress(Box::new(self.tree.clone()), Vec::new());
    }

    fn huff_compress(&mut self, node: Box<Node>, code: Vec<u8>) {
        match *node {
            Node::Internal(internal) => {
                if let Some(left_node) = internal.left {
                    let mut vec = Vec::from(code.clone());
                    vec.push(0);
                    self.huff_compress(left_node, vec);
                }

                if let Some(right_node) = internal.right {
                    let mut vec = Vec::from(code);
                    vec.push(1);
                    self.huff_compress(right_node, vec);
                }
            }

            Node::Leaf(leaf) => {
                self.char_codes.insert(leaf.symb, code);
            }
        }
    }

    fn get_compressed(&self) -> Vec<u8> {
        let mut result = Vec::new();

        for char in self.input.chars() {
            result.extend(self.char_codes.get(&char).unwrap());
        }

        result
    }

    fn decompress(&self, compressed: Vec<u8>) -> String {
        let mut result = String::new();
        let mut current_node = self.tree.clone();

        for val in compressed {
            if val == 0 {
                if let Node::Internal(internal) = current_node.clone() {
                    if let Some(left) = internal.left {
                        current_node = *left;
                    }
                }
            } else {
                if let Node::Internal(internal) = current_node.clone() {
                    if let Some(right) = internal.right {
                        current_node = *right;
                    }
                }
            }

            if let Node::Leaf(leaf) = current_node.clone() {
                result.push(leaf.symb);
                current_node = self.tree.clone();
            }
        }

        result
    }
}

impl From<Huffman> for NodeBytes {
    fn from(value: Huffman) -> Self {
        Self::from(value.tree)
    }
}

fn main() -> io::Result<()> {
    let mut args = args();

    args.next().unwrap();

    let Some(command) = args.next() else {
        return Err(Error::new(io::ErrorKind::InvalidInput, "Invalid arg"));
    };

    match command.as_str() {
        "compress" => {
            let Some(file_path) = args.next() else {
                return Err(Error::new(io::ErrorKind::InvalidInput, "Invalid file path"));
            };

            let Some(output_path) = args.next() else {
                return Err(Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid output file path",
                ));
            };

            let mut file = File::open(file_path)?;
            let mut buff = String::new();
            file.read_to_string(&mut buff)?;

            // Now we compress the data
            let mut huffman = Huffman::from_input(buff);

            huffman.compress();

            let mut bin = NodeBytes::from(huffman.clone());

            bin.as_bytes();

            let bytes = huffman
                .get_compressed()
                .iter()
                .map(|v| if *v == 1 as u8 { true } else { false })
                .collect::<Vec<bool>>();

            let mut bv: BitVec = BitVec::from_iter(bytes);

            let mut output = File::create_new(output_path)?;

            let mut written_bytes = 0;

            for byte in bin.bytes {
                if byte == 1 {
                    let mut tmp_b: BitVec = BitVec::from_iter([true]);
                    println!("{:?}", tmp_b);
                    let written = io::copy(&mut tmp_b, &mut output)?;
                    written_bytes += written;
                } else if byte == 0 {
                    let mut tmp_b: BitVec = BitVec::from_iter([false]);
                    println!("{:?}", tmp_b);
                    let written = io::copy(&mut tmp_b, &mut output)?;
                    written_bytes += written;
                } else {
                    let written = output.write(&[byte])?;
                    written_bytes += written as u64;
                }
            }
            output.write(&[0])?;
            let written = io::copy(&mut bv, &mut output)?;
            written_bytes += written;

            output.flush()?;

            println!("Compressed! {} bytes", written_bytes);
        }

        "decompress" => {
            let Some(file_path) = args.next() else {
                return Err(Error::new(io::ErrorKind::InvalidInput, "Invalid file path"));
            };

            let Some(output_path) = args.next() else {
                return Err(Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid output file path",
                ));
            };

            let mut compressed_file = File::open(file_path)?;

            let mut bv: BitVec = BitVec::new();

            io::copy(&mut compressed_file, &mut bv)?;


            println!("{:?}", bv);

        }

        c => {
            return Err(Error::new(
                io::ErrorKind::Other,
                format!("Commnad Not Found {}", c),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compress_huff() {
        let mut h = Huffman::from_input("Hello".to_owned());
        h.compress();

        assert_eq!(h.get_compressed(), vec![0, 0, 0, 1, 1, 1, 1, 1, 1, 0]);
    }

    #[test]
    fn decompress_huff() {
        let h = Huffman::from_input("Hello".to_owned());

        assert_eq!(h.decompress(vec![0, 0, 0, 1, 1, 1, 1, 1, 1, 0]), "Hello");
    }
}
