use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// A lightweight key referencing a value stored in a [ProvenanceMap](ProvenanceMap) or
/// [SeparateProvenanceMap](SeparateProvenanceMap).
///
/// Can only be created by methods on such map and thus will always be valid
/// for the map that created it. Further, the map that creates the key "tags"
/// it with it's provenance. And since there only may be one map with any given
/// provenance, it is guaranteed that if a key match the required type signature
/// for retrieving a value from a map, then that key were created by that map and
/// reference a value in that map.
pub struct Key<Provenance> {
    pub(crate) index: usize,
    _pd: PhantomData<Provenance>,
}

impl<Provenance> Key<Provenance> {
    /// Create a new key.
    ///
    /// Deliberately non-pub, since it should be created by calling methods
    /// on maps, which guarantee that the key is valid.
    pub(crate) fn new(index: usize) -> Self {
        Key {
            index,
            _pd: Default::default(),
        }
    }
}

// Deriving traits for Key has proved unreliable, hence they are manually implemented.

impl<Provenance> Debug for Key<Provenance> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Key({})", self.index)
    }
}

// Clone + Copy

impl<Provenance> Clone for Key<Provenance> {
    fn clone(&self) -> Self {
        Key {
            index: self.index,
            _pd: Default::default(),
        }
    }
}

impl<Provenance> Copy for Key<Provenance> {}

// PartialEq + Eq

impl<Provenance> PartialEq for Key<Provenance> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<Provenance> Eq for Key<Provenance> {}

// Hash

impl<Provenance> Hash for Key<Provenance> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state)
    }
}
