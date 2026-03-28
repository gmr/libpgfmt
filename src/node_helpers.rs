use tree_sitter::Node;

/// Extension trait for tree-sitter nodes.
pub(crate) trait NodeExt<'a> {
    /// Get the source text for this node.
    fn text(&self, source: &'a str) -> &'a str;

    /// Find the first named child with the given kind.
    fn find_child(&self, kind: &str) -> Option<Node<'a>>;

    /// Find first child matching any of the given kinds.
    fn find_child_any(&self, kinds: &[&str]) -> Option<Node<'a>>;

    /// Get all named children.
    fn named_children_vec(&self) -> Vec<Node<'a>>;

    /// Check if this node has a named child with the given kind.
    fn has_child(&self, kind: &str) -> bool;
}

impl<'a> NodeExt<'a> for Node<'a> {
    fn text(&self, source: &'a str) -> &'a str {
        &source[self.byte_range()]
    }

    fn find_child(&self, kind: &str) -> Option<Node<'a>> {
        let mut cursor = self.walk();
        self.named_children(&mut cursor)
            .find(|&child| child.kind() == kind)
    }

    fn find_child_any(&self, kinds: &[&str]) -> Option<Node<'a>> {
        let mut cursor = self.walk();
        self.named_children(&mut cursor)
            .find(|&child| kinds.contains(&child.kind()))
    }

    fn named_children_vec(&self) -> Vec<Node<'a>> {
        let mut cursor = self.walk();
        self.named_children(&mut cursor).collect()
    }

    fn has_child(&self, kind: &str) -> bool {
        self.find_child(kind).is_some()
    }
}

/// Flatten a left-recursive list node (like `target_list`, `expr_list`, etc.)
/// into a vector of the leaf items.
///
/// In the tree-sitter-postgres grammar, lists are encoded as left-recursive rules:
///   target_list -> target_list ',' target_el | target_el
///
/// This function collects all the non-list leaf items.
pub(crate) fn flatten_list<'a>(node: Node<'a>, list_kind: &str) -> Vec<Node<'a>> {
    let mut items = Vec::new();
    flatten_list_inner(node, list_kind, &mut items);
    items
}

fn flatten_list_inner<'a>(node: Node<'a>, list_kind: &str, items: &mut Vec<Node<'a>>) {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if child.kind() == list_kind {
            flatten_list_inner(child, list_kind, items);
        } else {
            items.push(child);
        }
    }
}
