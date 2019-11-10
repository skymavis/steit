use std::{io, rc::Rc};

use crate::{wire_type::WireType, Serialize};

use super::{
    log::{Entry, Logger},
    node::Node,
};
use crate::rt::log::EntryKind;

#[derive(Default, Debug)]
pub struct Runtime {
    logger: Logger,
    path: Rc<Node<u16>>,
}

impl Runtime {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn nested(&self, tag: u16) -> Self {
        Self {
            logger: self.logger.clone(),
            path: Rc::new(Node::child(&self.path, tag)),
        }
    }

    #[inline]
    pub fn parent(&self) -> Self {
        Self {
            logger: self.logger.clone(),
            path: self.path.parent().expect("expect a parent `Runtime`"),
        }
    }

    #[inline]
    pub fn log_update(&self, tag: u16, value: &impl Serialize) -> io::Result<()> {
        self.logger
            .log_entry(Entry::new(&self.nested(tag), EntryKind::Update { value }))
    }

    #[inline]
    pub fn log_update_in_place(&self, value: &impl Serialize) -> io::Result<()> {
        self.logger
            .log_entry(Entry::new(self, EntryKind::Update { value }))
    }

    #[inline]
    pub fn log_add(&self, item: &impl Serialize) -> io::Result<()> {
        self.logger
            .log_entry(Entry::new(self, EntryKind::Add { item }))
    }

    #[inline]
    pub fn log_remove<T: Serialize>(&self, tag: u16) -> io::Result<()> {
        self.logger
            .log_entry(Entry::<T>::new(&self.nested(tag), EntryKind::Remove))
    }

    #[inline]
    pub fn get_or_set_cached_size_from(&self, f: impl FnOnce() -> u32) -> u32 {
        match &*self.path {
            Node::Root { cached_size } => cached_size.get_or_set_from(f),
            Node::Child { cached_size, .. } => cached_size.get_or_set_from(f),
        }
    }

    #[inline]
    pub fn clear_cached_size(&self) {
        Self::clear_cached_size_branch(&self.path);
    }

    fn clear_cached_size_branch(node: &Node<u16>) {
        match node {
            Node::Root { cached_size } => cached_size.clear(),
            Node::Child {
                parent,
                cached_size,
                ..
            } => {
                cached_size.clear();
                Self::clear_cached_size_branch(parent);
            }
        }
    }
}

impl PartialEq for Runtime {
    #[inline]
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for Runtime {}

impl WireType for Runtime {
    const WIRE_TYPE: u8 = <Node<u16> as WireType>::WIRE_TYPE;
}

impl Serialize for Runtime {
    #[inline]
    fn size(&self) -> u32 {
        self.path.size()
    }

    #[inline]
    fn serialize(&self, writer: &mut impl io::Write) -> io::Result<()> {
        self.path.serialize(writer)
    }
}

#[cfg(test)]
mod tests {
    use iowrap::Eof;

    use crate::{
        rt::{node::Node, path::Path},
        Deserialize, Serialize,
    };

    use super::Runtime;

    #[test]
    fn serialization() {
        let runtime = Runtime::new().nested(10).nested(20);
        let mut bytes = Vec::new();

        runtime.serialize(&mut bytes).unwrap();

        let path = Path::deserialize(&mut Eof::new(&*bytes)).unwrap();

        assert_eq!(&*path, &[10, 20]);
    }

    #[test]
    fn clear_cached_size_branch() {
        // 2 level deep `Runtime`
        let runtime = Runtime::new().nested(2);

        // Set cached sizes of both `Runtime` nodes
        match &*runtime.path {
            Node::Root { .. } => assert!(false),
            Node::Child {
                parent,
                cached_size,
                ..
            } => {
                cached_size.set(7);

                match &**parent {
                    Node::Root { cached_size } => cached_size.set(6),
                    Node::Child { .. } => assert!(false),
                }
            }
        }

        runtime.parent().clear_cached_size();

        match &*runtime.path {
            Node::Root { .. } => assert!(false),
            Node::Child {
                parent,
                cached_size,
                ..
            } => {
                // Cached size of the leaf `Runtime` is still set.
                assert!(cached_size.is_set());

                match &**parent {
                    // Cached size of the root `Runtime` has been cleared.
                    Node::Root { cached_size } => assert!(!cached_size.is_set()),
                    Node::Child { .. } => assert!(false),
                }
            }
        };

        runtime.clear_cached_size();

        match &*runtime.path {
            Node::Root { .. } => assert!(false),
            Node::Child {
                parent,
                cached_size,
                ..
            } => {
                // Now cached size of the leaf runtime has also been cleared.
                assert!(!cached_size.is_set());

                match &**parent {
                    Node::Root { cached_size } => assert!(!cached_size.is_set()),
                    Node::Child { .. } => assert!(false),
                }
            }
        };
    }
}