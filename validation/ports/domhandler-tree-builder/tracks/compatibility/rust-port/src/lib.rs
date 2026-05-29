#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeId(pub usize);

#[derive(Debug, PartialEq, Eq)]
pub enum NodeKind {
    Root,
    Element(String),
    Text(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Node {
    pub kind: NodeKind,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BuildError {
    UnexpectedCloseTag,
}

pub struct DomHandler {
    nodes: Vec<Node>,
    stack: Vec<NodeId>,
}

impl DomHandler {
    pub fn new() -> Self {
        Self {
            nodes: vec![Node {
                kind: NodeKind::Root,
                parent: None,
                children: Vec::new(),
            }],
            stack: vec![NodeId(0)],
        }
    }

    pub fn on_open_tag(&mut self, name: &str) -> NodeId {
        let parent_id = *self.stack.last().expect("root is always open");
        let child_id = self.push_node(NodeKind::Element(name.to_owned()), Some(parent_id));
        self.nodes[parent_id.0].children.push(child_id);
        self.stack.push(child_id);
        child_id
    }

    pub fn on_text(&mut self, value: &str) -> NodeId {
        let parent_id = *self.stack.last().expect("root is always open");
        let child_id = self.push_node(NodeKind::Text(value.to_owned()), Some(parent_id));
        self.nodes[parent_id.0].children.push(child_id);
        child_id
    }

    pub fn on_close_tag(&mut self, _name: &str) -> Result<(), BuildError> {
        if self.stack.len() <= 1 {
            return Err(BuildError::UnexpectedCloseTag);
        }

        self.stack.pop();
        Ok(())
    }

    pub fn root_children(&self) -> &[NodeId] {
        &self.nodes[0].children
    }

    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.0]
    }

    fn push_node(&mut self, kind: NodeKind, parent: Option<NodeId>) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Node {
            kind,
            parent,
            children: Vec::new(),
        });
        id
    }
}

impl Default for DomHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compatibility_builds_element_with_text_child() {
        let mut handler = DomHandler::new();

        let root = handler.on_open_tag("root");
        let text = handler.on_text("hello");
        handler.on_close_tag("root").unwrap();

        assert_eq!(handler.root_children(), &[root]);
        assert_eq!(handler.node(root).parent, Some(NodeId(0)));
        assert_eq!(handler.node(root).children, vec![text]);
        assert_eq!(handler.node(text).parent, Some(root));
        assert_eq!(handler.node(text).kind, NodeKind::Text("hello".to_owned()));
    }

    #[test]
    fn compatibility_builds_nested_elements() {
        let mut handler = DomHandler::new();

        let section = handler.on_open_tag("section");
        let item = handler.on_open_tag("item");
        handler.on_text("value");
        handler.on_close_tag("item").unwrap();
        handler.on_close_tag("section").unwrap();

        assert_eq!(handler.root_children(), &[section]);
        assert_eq!(handler.node(section).children, vec![item]);
        assert_eq!(handler.node(item).parent, Some(section));
    }

    #[test]
    fn compatibility_rejects_extra_close_tag() {
        let mut handler = DomHandler::new();

        assert_eq!(
            handler.on_close_tag("root"),
            Err(BuildError::UnexpectedCloseTag)
        );
    }
}
