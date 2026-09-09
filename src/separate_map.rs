use crate::{Key, SeparateProvenanceMapTransformer};
use std::any::TypeId;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::ops::DerefMut;
use std::sync::{LazyLock, Mutex};

/// A [ProvenanceMap](ProvenanceMap) where the a type separate from the type of the stored
/// values can be used to signify the maps provenance.
///
/// This allows for multiple maps that store the values of the same type to be created,
/// as long as the type given for the `Provenance` parameter is unique.
/// ```
/// use provenance::{ProvenanceMap, SeparateProvenanceMap};
///
/// // Structs that only exist to denote provenance
/// struct One; struct Two;
///
/// // Creating a map once is OK
/// let mut map = SeparateProvenanceMap::<One, String>::new();
/// assert!(map.is_some());
///
/// // Creating another map is OK as long as the provenance is different
/// let mut map = SeparateProvenanceMap::<Two, String>::new();
/// assert!(map.is_some());
/// ```
///
/// A [ProvenanceMap](ProvenanceMap) can be thought of as a special case of this map
/// where the type of the stored values also is used as provenance. That is
/// `ProvenanceMap<i32> ≈ SeparateProvenanceMap<i32, i32>`. Currently, this is how
/// [ProvenanceMap](ProvenanceMap) is implemented. This has the effect that both
/// types share the available pool of types that can be used as provenance. That
/// means that both a `ProvenanceMap<i32>` and a `SeperateProvenanceMap<i32, B>`
/// for any type `B` can not be constructed. This beahviour may change in future
/// versions and should not be relied upon.
/// ```
/// use provenance::{ProvenanceMap, SeparateProvenanceMap};
///
/// // Creating a map once is OK
/// let mut map = ProvenanceMap::<i32>::new();
/// assert!(map.is_some());
///
/// // Creating another map with the same provenance as another
/// // is not OK, even if that map is a ProvenanceMap
/// let mut map = SeparateProvenanceMap::<i32, bool>::new();
/// assert!(map.is_none());
/// ```
pub struct SeparateProvenanceMap<Provenance, Value> {
    elements: Vec<Value>,
    _pd: PhantomData<Provenance>,
}

impl<Provenance: 'static, Value> SeparateProvenanceMap<Provenance, Value> {
    /// Creates a new empty map with some type as provenance.
    ///
    /// If a map with such provenance already has been created, [`None`](std::option::Option::None) will be returned.
    /// Thus be careful to not drop maps unintentionally.
    /// ```
    /// use provenance::SeparateProvenanceMap;
    ///
    /// struct Provenance;
    ///
    /// // Creating a map with some type as provenance is ok
    /// let map = SeparateProvenanceMap::<Provenance, bool>::new();
    /// assert!(map.is_some());
    ///
    /// // Creating another is not however
    /// let map = SeparateProvenanceMap::<Provenance, i32>::new();
    /// assert!(map.is_none());
    /// ```
    pub fn new() -> Option<SeparateProvenanceMap<Provenance, Value>> {
        static USED_PROVENANCE: LazyLock<Mutex<HashSet<TypeId>>> =
            LazyLock::new(|| Mutex::new(Default::default()));

        let mut lock = USED_PROVENANCE
            .lock()
            .expect("USED_PROVENANCE mutex should never be poisoned");
        let used_maps = lock.deref_mut();

        let type_id = TypeId::of::<Provenance>();
        let is_first_use = used_maps.insert(type_id);

        if !is_first_use {
            return None;
        }

        Some(SeparateProvenanceMap {
            elements: vec![],
            _pd: Default::default(),
        })
    }

    /// Insert a value into this map.
    /// A unique key is returned. The key may be used to retrieve the value.
    /// ```
    /// use provenance::SeparateProvenanceMap;
    /// struct Provenance;
    /// let mut map = SeparateProvenanceMap::<Provenance, i32>::new().unwrap();
    ///
    /// let key = map.insert(5);
    /// assert_eq!(&5, map.get(key));
    /// ```
    ///
    /// Values are not required to be unique in the map.
    /// ```
    /// use provenance::SeparateProvenanceMap;
    /// struct Provenance;
    /// let mut map = SeparateProvenanceMap::<Provenance, i32>::new().unwrap();
    ///
    /// let key1 = map.insert(5);
    /// let key2 = map.insert(5);
    /// assert_ne!(key1, key2);
    /// ```
    pub fn insert(&mut self, value: Value) -> Key<Provenance> {
        let index = self.elements.len();
        self.elements.insert(index, value);
        Key::new(index)
    }

    /// Use a [key](Key) to retrieve an immutable reference to a stored value.
    /// ```
    /// use provenance::SeparateProvenanceMap;
    /// struct Provenance;
    /// let mut map = SeparateProvenanceMap::<Provenance, i32>::new().unwrap();
    ///
    /// let key = map.insert(5);
    /// assert_eq!(&5, map.get(key));
    /// ```
    pub fn get(&self, key: Key<Provenance>) -> &Value {
        // The key has the correct provenance,
        // thus we know that we created it in `insert`,
        // thus it is safe to use.
        &self.elements[key.index]
    }

    /// Use a [key](Key) to retrieve a mutable reference to a stored value.
    /// ```
    /// use provenance::SeparateProvenanceMap;
    /// struct Provenance;
    /// let mut map = SeparateProvenanceMap::<Provenance, i32>::new().unwrap();
    ///
    /// let key = map.insert(5);
    /// assert_eq!(&mut 5, map.get_mut(key));
    /// ```
    pub fn get_mut(&mut self, key: Key<Provenance>) -> &mut Value {
        // The key has the correct provenance,
        // thus we know that we created it in `insert`,
        // thus it is safe to use.
        &mut self.elements[key.index]
    }

    /// Get an [iterator](Iterator) over all entries in the map.
    /// ```
    /// use provenance::SeparateProvenanceMap;
    /// struct Provenance;
    /// let mut map = SeparateProvenanceMap::<Provenance, i32>::new().unwrap();
    ///
    /// let key1 = map.insert(1);
    /// let key2 = map.insert(2);
    /// let key3 = map.insert(3);
    ///
    /// let mut entries = map.entries();
    /// assert_eq!(Some((key1, &1)), entries.next());
    /// assert_eq!(Some((key2, &2)), entries.next());
    /// assert_eq!(Some((key3, &3)), entries.next());
    /// assert_eq!(None, entries.next());
    /// ```
    pub fn entries(&self) -> impl Iterator<Item = (Key<Provenance>, &Value)> {
        self.elements
            .iter()
            .enumerate()
            .map(|(index, value)| (Key::new(index), value))
    }

    /// Get an [iterator](Iterator) over mutable references to all entries in the map.
    /// ```
    /// use provenance::SeparateProvenanceMap;
    /// struct Provenance;
    /// let mut map = SeparateProvenanceMap::<Provenance, i32>::new().unwrap();
    ///
    /// let key1 = map.insert(1);
    /// let key2 = map.insert(2);
    /// let key3 = map.insert(3);
    ///
    /// let mut entries = map.entries_mut();
    /// assert_eq!(Some((key1, &mut 1)), entries.next());
    /// assert_eq!(Some((key2, &mut 2)), entries.next());
    /// assert_eq!(Some((key3, &mut 3)), entries.next());
    /// assert_eq!(None, entries.next());
    /// ```
    pub fn entries_mut(&mut self) -> impl Iterator<Item = (Key<Provenance>, &mut Value)> {
        self.elements
            .iter_mut()
            .enumerate()
            .map(|(index, value)| (Key::new(index), value))
    }

    /// Get an [iterator](Iterator) over all keys in the map.
    /// ```
    /// use provenance::SeparateProvenanceMap;
    /// struct Provenance;
    /// let mut map = SeparateProvenanceMap::<Provenance, i32>::new().unwrap();
    ///
    /// map.insert(1);
    /// map.insert(2);
    /// map.insert(3);
    ///
    /// assert_eq!(3, map.keys().count());
    /// ```
    pub fn keys(&self) -> impl Iterator<Item = Key<Provenance>> {
        self.entries().map(|(k, _)| k)
    }

    /// Get an [iterator](Iterator) over immutable references to each value in the map.
    /// ```
    /// use provenance::SeparateProvenanceMap;
    /// struct Provenance;
    /// let mut map = SeparateProvenanceMap::<Provenance, i32>::new().unwrap();
    ///
    /// map.insert(1);
    /// map.insert(2);
    /// map.insert(3);
    ///
    /// assert_eq!(6, map.iter().sum());
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &Value> {
        self.elements.iter()
    }

    /// Get an [iterator](Iterator) over mutable references to each value in the map.
    /// ```
    /// use provenance::SeparateProvenanceMap;
    /// struct Provenance;
    /// let mut map = SeparateProvenanceMap::<Provenance, i32>::new().unwrap();
    ///
    /// map.insert(1);
    /// map.insert(2);
    /// map.insert(3);
    ///
    /// // Add one to every value
    /// map.iter_mut().for_each(|val| *val += 1);
    ///
    /// assert_eq!(9, map.iter().sum());
    /// ```
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Value> {
        self.elements.iter_mut()
    }

    /// Search the map in insertion order for the first value that satisfy the given predicate.
    /// If such value is found, an immutable reference to it is returned,
    /// ```
    /// use provenance::SeparateProvenanceMap;
    /// struct Provenance;
    /// let mut map = SeparateProvenanceMap::<Provenance, i32>::new().unwrap();
    ///
    /// map.insert(1);
    /// map.insert(2);
    /// map.insert(3);
    ///
    /// assert_eq!(Some(&2), map.find(|&val| val == 2));
    /// ```
    /// otherwise [`None`](std::option::Option::None) is returned.
    /// ```
    /// use provenance::SeparateProvenanceMap;
    /// struct Provenance;
    /// let mut map = SeparateProvenanceMap::<Provenance, i32>::new().unwrap();
    ///
    /// map.insert(1);
    /// map.insert(2);
    /// map.insert(3);
    ///
    /// assert_eq!(None, map.find(|&val| val == 53));
    /// ```
    pub fn find<P: Fn(&Value) -> bool>(&self, predicate: P) -> Option<&Value> {
        for value in self.elements.iter() {
            if predicate(value) {
                return Some(value);
            }
        }

        None
    }

    /// Search the map in insertion order for the first value that satisfy the given predicate.
    /// If such value is found, a mutable reference to it is returned,
    /// ```
    /// use provenance::SeparateProvenanceMap;
    /// struct Provenance;
    /// let mut map = SeparateProvenanceMap::<Provenance, i32>::new().unwrap();
    ///
    /// map.insert(1);
    /// map.insert(2);
    /// map.insert(3);
    ///
    /// assert_eq!(Some(&mut 2), map.find_mut(|&val| val == 2));
    /// ```
    /// otherwise [`None`](std::option::Option::None) is returned.
    /// ```
    /// use provenance::SeparateProvenanceMap;
    /// struct Provenance;
    /// let mut map = SeparateProvenanceMap::<Provenance, i32>::new().unwrap();
    ///
    /// map.insert(1);
    /// map.insert(2);
    /// map.insert(3);
    ///
    /// assert_eq!(None, map.find_mut(|&val| val == 53));
    /// ```
    pub fn find_mut<P: Fn(&Value) -> bool>(&mut self, predicate: P) -> Option<&mut Value> {
        for value in self.elements.iter_mut() {
            if predicate(value) {
                return Some(value);
            }
        }

        None
    }

    pub fn transform(&self) -> SeparateProvenanceMapTransformer<'_, Provenance, Value> {
        SeparateProvenanceMapTransformer::new_from(self)
    }
}
