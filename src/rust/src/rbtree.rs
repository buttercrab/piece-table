use std::mem;

use bitflags::bitflags;
use ghost_cell::{GhostCell, GhostToken};
use static_rc::StaticRc;

bitflags! {
    /// Node Flags
    struct NodeFlags: u8 {
        const RED = 0b000;
        const BLACK = 0b001;
        const HAS_LEFT = 0b010;
        const HAS_RIGHT = 0b100;
    }
}

/// # Node
///
/// Simple Node for indexed RBTree
struct Node<'a, T> {
    value: T,
    flags: NodeFlags,
    size: usize,
    weight: usize,
    parent: Option<StaticRc<GhostCell<'a, Node<'a, T>>, 1, 3>>,
    left: Option<StaticRc<GhostCell<'a, Node<'a, T>>, 1, 3>>,
    right: Option<StaticRc<GhostCell<'a, Node<'a, T>>, 1, 3>>,
}

impl<'a, T> Node<'a, T> {
    #[inline]
    fn new(
        value: T,
        weight: usize,
        token: &mut GhostToken<'a>,
    ) -> StaticRc<GhostCell<'a, Node<'a, T>>, 1, 3> {
        let node = Node {
            value,
            flags: NodeFlags::empty(),
            size: weight,
            weight,
            parent: None,
            left: None,
            right: None,
        };

        let node = StaticRc::new(GhostCell::new(node));
        let (parent, children) = StaticRc::split::<1, 2>(node);
        let (left, right) = StaticRc::split::<1, 1>(children);

        parent.borrow_mut(token).left = Some(left);
        parent.borrow_mut(token).right = Some(right);
        parent
    }

    #[inline]
    fn is_black(&self) -> bool {
        self.flags.contains(NodeFlags::BLACK)
    }

    #[inline]
    fn left(&self) -> Option<&StaticRc<GhostCell<'a, Node<'a, T>>, 1, 3>> {
        if self.flags.contains(NodeFlags::HAS_LEFT) {
            self.left.as_ref()
        } else {
            None
        }
    }

    #[inline]
    fn set_left(
        &mut self,
        new_node: StaticRc<GhostCell<'a, Node<'a, T>>, 1, 3>,
    ) -> StaticRc<GhostCell<'a, Node<'a, T>>, 1, 3> {
        self.left.replace(new_node).unwrap()
    }

    #[inline]
    fn right(&self) -> Option<&StaticRc<GhostCell<'a, Node<'a, T>>, 1, 3>> {
        if self.flags.contains(NodeFlags::HAS_RIGHT) {
            self.right.as_ref()
        } else {
            None
        }
    }

    #[inline]
    fn set_right(
        &mut self,
        new_node: StaticRc<GhostCell<'a, Node<'a, T>>, 1, 3>,
    ) -> StaticRc<GhostCell<'a, Node<'a, T>>, 1, 3> {
        self.right.replace(new_node).unwrap()
    }

    /// Assertion: parent, left, right is the three part of this node
    #[inline]
    fn join_rc(&mut self) -> StaticRc<GhostCell<'a, Node<'a, T>>, 3, 3> {
        let p = self.parent.take().unwrap();
        let l = self.left.take().unwrap();
        let r = self.right.take().unwrap();

        // SAFETY: by assertion, `l`, `r`, and `p` are same pointers
        let c: StaticRc<GhostCell<'a, Node<'a, T>>, 2, 3> =
            unsafe { StaticRc::join_unchecked(l, r) };
        unsafe { StaticRc::join_unchecked(p, c) }
    }

    /// Assertion: `parent` do not have left node
    ///            or `parent`'s left is `left`
    #[inline]
    fn toggle_left(
        parent: &StaticRc<GhostCell<'a, Node<'a, T>>, 1, 3>,
        left: &StaticRc<GhostCell<'a, Node<'a, T>>, 1, 3>,
        _: &mut GhostToken<'a>,
    ) {
        // SAFETY: `p` and `l` just sawp each element.
        //         Although `GhostToken` prevents only one modification
        //         at one time, they don't collide each other.
        //         Also, by having `&mut GhostToken` in param,
        //         this is the only modification.
        let p = unsafe { &mut *parent.as_ptr() };
        let l = unsafe { &mut *left.as_ptr() };
        mem::swap(&mut p.left, &mut l.parent);
        p.flags.toggle(NodeFlags::HAS_LEFT);
    }

    /// Assertion: `parent` do not have right node
    ///            or `parent`'s right is `right`
    #[inline]
    fn toggle_right(
        parent: &StaticRc<GhostCell<'a, Node<'a, T>>, 1, 3>,
        right: &StaticRc<GhostCell<'a, Node<'a, T>>, 1, 3>,
        _: &mut GhostToken<'a>,
    ) {
        // SAFETY: Same as `toggle_left`.
        let p = unsafe { &mut *parent.as_ptr() };
        let r = unsafe { &mut *right.as_ptr() };
        mem::swap(&mut p.right, &mut r.parent);
        p.flags.toggle(NodeFlags::HAS_RIGHT);
    }

    fn drop_recursive(
        node: &StaticRc<GhostCell<'a, Node<'a, T>>, 1, 3>,
        token: &mut GhostToken<'a>,
    ) {
        todo!()
    }
}

pub struct IndexedRBTree<'a, T> {
    len: usize,
    root: Option<StaticRc<GhostCell<'a, Node<'a, T>>, 1, 3>>,
}

impl<'a, T> IndexedRBTree<'a, T> {
    pub fn new() -> IndexedRBTree<'a, T> {
        IndexedRBTree { len: 0, root: None }
    }

    pub fn singleton(value: T, weight: usize, token: &mut GhostToken<'a>) -> IndexedRBTree<'a, T> {
        IndexedRBTree {
            len: 1,
            root: Some(Node::new(value, weight, token)),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn front<'t>(&'t self, token: &'t GhostToken<'a>) -> Option<&'t T> {
        self.root.as_ref().map(|root| {
            let mut root = root.borrow(token);

            while let Some(l) = root.left() {
                root = l.borrow(token);
            }

            &root.value
        })
    }

    pub fn back<'t>(&'t self, token: &'t GhostToken<'a>) -> Option<&'t T> {
        self.root.as_ref().map(|root| {
            let mut root = root.borrow(token);

            while let Some(r) = root.right() {
                root = r.borrow(token);
            }

            &root.value
        })
    }

    pub fn clear(&mut self, token: &mut GhostToken<'a>) {
        todo!()
    }

    fn rotate_left(
        &mut self,
        node: &StaticRc<GhostCell<'a, Node<'a, T>>, 1, 3>,
        token: &mut GhostToken<'a>,
    ) {
        todo!()
    }

    fn rotate_right(
        &mut self,
        node: &StaticRc<GhostCell<'a, Node<'a, T>>, 1, 3>,
        token: &mut GhostToken<'a>,
    ) {
        todo!()
    }

    pub fn insert(&mut self, value: T, index: usize, token: &mut GhostToken<'a>) {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_test() {
        let tree: IndexedRBTree<usize> = IndexedRBTree::new();
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.is_empty(), true);
    }

    #[test]
    fn singleton_test() {
        GhostToken::new(|mut token| {
            let tree = IndexedRBTree::singleton(0, 0, &mut token);
            assert_eq!(tree.len(), 1);
            assert_eq!(tree.is_empty(), false);

            // assert_eq!(tree.front(&token), Some(&1));
            // assert_eq!(tree.back(&token), Some(&1));
        });
    }
}
