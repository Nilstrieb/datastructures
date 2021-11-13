use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
struct BinaryTree<T>(Node<T>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node<T> {
    lhs: Option<Box<Node<T>>>,
    val: T,
    rhs: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new(value: T, lhs: Option<Node<T>>, rhs: Option<Node<T>>) -> Self {
        Self {
            lhs: lhs.map(Box::new),
            val: value,
            rhs: rhs.map(Box::new),
        }
    }

    pub fn leaf(value: T) -> Self {
        Self::new(value, None, None)
    }
}

pub trait DisplayTree {
    fn depth(&self) -> usize;
    fn offset_x(&self) -> usize;
    fn amount_of_con(&self) -> usize;
    fn display(&self) -> String;
}

impl<T: Display> DisplayTree for Node<T> {
    fn depth(&self) -> usize {
        self.lhs
            .as_ref()
            .map(|node| node.depth() + 1)
            .unwrap_or(0)
            .max(self.rhs.as_ref().map(|node| node.depth() + 1).unwrap_or(0))
    }

    fn offset_x(&self) -> usize {
        let offset_below = self.lhs.as_ref().map(|node| node.offset_x()).unwrap_or(0);
        let depth = self.depth();

        if depth == 0 {
            return 0;
        }

        offset_below + self.amount_of_con() + 1
    }

    fn amount_of_con(&self) -> usize {
        let depth = self.depth();
        if depth == 0 {
            0
        } else {
            2_usize.pow((depth - 1) as u32)
        }
    }

    fn display(&self) -> String {
        let mut str = String::new();

        let offset_x = self.offset_x();

        // top level

        str.push_str(&".".repeat(offset_x));
        str.push_str(&self.val.to_string());
        str.push('\n');

        for i in 0..self.amount_of_con() {
            str.push_str(&".".repeat(offset_x - i - 1));
            str.push('/');
            str.push_str(&".".repeat(1 + 2 * i));
            str.push('\\');
            str.push('\n');
        }

        // rest
        // let mut next_nodes = vec![];
        let mut current_nodes = vec![&self.lhs, &self.rhs]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        while current_nodes.len() > 0 {
            // display node layer

            for (i, node) in current_nodes.iter().enumerate() {
                // print node values
                str.push_str(&".".repeat(node.offset_x()));
                str.push_str(&node.val.to_string());
                if i != current_nodes.len() - 1 {
                    str.push_str(&".".repeat(node.offset_x() + 1));
                }
            }
            str.push('\n');
            // print node connections

            let mut offset = 0;
            let nodes_with_offset = current_nodes
                .iter()
                .map(|node| {
                    offset += node.offset_x();
                    let this_offset = offset;
                    offset += node.val.to_string().len();
                    offset += node.offset_x() + 1;
                    (this_offset, node)
                })
                .collect::<Vec<_>>();

            let amount_of_con = current_nodes
                .first()
                .map(|node| node.amount_of_con())
                .unwrap_or(0);

            for i in 0..amount_of_con {
                let mut connections = nodes_with_offset
                    .iter()
                    .map(|(offset, _)| (offset - 1 - i, '/'))
                    .chain(
                        nodes_with_offset
                            .iter()
                            .map(|(offset, _)| (offset + 1 + i, '\\')),
                    )
                    .collect::<Vec<_>>();
                connections.sort_by(|(a_offset, _), (b_offset, _)| a_offset.cmp(b_offset));

                let mut prev_offset = 0;
                for (offset, con) in connections {
                    let diff_offset = offset - prev_offset;
                    str.push_str(&".".repeat(diff_offset));
                    str.push(con);
                    prev_offset += diff_offset + 1;
                }
                str.push('\n');
            }

            current_nodes = current_nodes
                .into_iter()
                .map(|node| [&node.lhs, &node.rhs])
                .flatten()
                .flatten()
                .collect();
        }

        str
    }
}

mod test {
    use crate::binary_tree::{DisplayTree, Node};

    #[test]
    fn print_cool_tree() {
        let tree = Node::new(
            4,
            Some(Node::new(2, Some(Node::leaf(1)), Some(Node::leaf(3)))),
            Some(Node::new(6, Some(Node::leaf(5)), Some(Node::leaf(7)))),
        );

        println!("{}", tree.display());

        panic!("fail");
    }
}
