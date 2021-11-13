use std::fmt::{Debug, Display};

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

impl<T: Display + Debug> DisplayTree for Node<T> {
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
        fn amount(n: usize) -> usize {
            match n {
                0 => 0,
                2 => 2,
                n => amount(n - 1) * 2 + 1,
            }
        }

        amount(self.depth())
    }

    fn display(&self) -> String {
        const SPACE: &str = " ";

        let mut str = String::new();

        let mut current_nodes = vec![self];

        while current_nodes.len() > 0 {
            // display node layer

            let mut offset = 0;
            let mut is_left = true;
            let nodes_with_offset = current_nodes
                .iter()
                .map(|node| {
                    offset += node.offset_x();
                    let this_offset = offset;
                    offset += node.val.to_string().len();
                    offset += node.offset_x() + 1;
                    if node.depth() == 0 && is_left {
                        offset += 2;
                    }
                    is_left = !is_left;
                    (this_offset, node)
                })
                .collect::<Vec<_>>();

            let mut prev_offset = 0;
            for (offset, node) in &nodes_with_offset {
                let diff_offset = offset - prev_offset;
                str.push_str(&SPACE.repeat(diff_offset));
                let value_str = node.val.to_string();
                str.push_str(&value_str);
                prev_offset += diff_offset + value_str.len();
            }
            str.push('\n');
            // print node connections

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
                    str.push_str(&SPACE.repeat(diff_offset));
                    str.push(con);
                    prev_offset += diff_offset + 1;
                }
                str.push('\n');
            }

            current_nodes = current_nodes
                .iter()
                .map(|node| [&node.lhs, &node.rhs])
                .flatten()
                .flatten()
                .map(|boxed| &**boxed)
                .collect::<Vec<_>>();
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
        let cooler_tree = Node::new(5, Some(tree.clone()), Some(tree.clone()));
        println!("{}", cooler_tree.display());

        let epic_tree = Node::new(5, Some(cooler_tree.clone()), Some(cooler_tree.clone()));
        println!("{}", epic_tree.display());

        let giant_tree = Node::new(5, Some(epic_tree.clone()), Some(epic_tree.clone()));
        println!("{}", giant_tree.display());

        panic!("fail");
    }
}
