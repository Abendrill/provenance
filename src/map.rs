use crate::{Key, ProvenanceMapTransformer, SeparateProvenanceMap};

/// A provenance map is a map-like data structure that know which keys belong
/// to which map.
///
/// Keys are generated upon inserting an element into the map.
/// ```
/// use provenance::ProvenanceMap;
/// let mut map = ProvenanceMap::<i32>::new().unwrap();
/// let key = map.insert(5);
/// ```
///
/// This is achieved by "tagging" keys with the generic type parameter of the map. So
/// a map of type `ProvenanceMap<i32>` will create keys of type `Key<i32>`. The key
/// does not actually contain a value of their generic type parameter. It is only used
/// to track what map the key came from, i.e. it's provenance.
/// ```compile_fail
/// use provenance::ProvenanceMap;
/// let mut map_1 = ProvenanceMap::<i32>::new().unwrap();
/// let key = map_1.insert(5);
/// let map_2 = ProvenanceMap::<bool>::new().unwrap();
///
/// // Using a key from another map is a type error.
/// map_2.get(key);
/// ```
///
/// However, if it were possible to create multiple maps with the same type, e.g.
/// `ProvenanceMap<String>` the type of the key wouldn't be enough to track what map
/// a key came from. Therefore, it is only possible to create a single map per
/// concrete type given for the `Value` type parameter.
/// ```
/// use provenance::ProvenanceMap;
///
/// // Creating a map once is OK
/// let mut map = ProvenanceMap::<String>::new();
/// assert!(map.is_some());
///
/// // Creating another map with the same signature is not OK
/// let mut map = ProvenanceMap::<String>::new();
/// assert!(map.is_none());
/// ```
///
pub struct ProvenanceMap<Value> {
    pub(crate) inner: SeparateProvenanceMap<Value, Value>,
}

impl<Value: 'static> ProvenanceMap<Value> {
    /// Create a new map if one with the given signature have not already been created.
    /// If one has, [`None`](std::option::Option::None) is returned.
    /// ```
    /// use provenance::ProvenanceMap;
    ///
    /// // Creating a map once is OK
    /// let map = ProvenanceMap::<String>::new();
    /// assert!(map.is_some());
    ///
    /// // Creating another map with the same signature is not OK
    /// let map = ProvenanceMap::<String>::new();
    /// assert!(map.is_none());
    /// ```
    pub fn new() -> Option<ProvenanceMap<Value>> {
        let map = SeparateProvenanceMap::new()?;

        Some(ProvenanceMap { inner: map })
    }

    /// Insert a value into the map.
    /// A key is generated for the value and returned.
    /// This key can be used to access the value later.
    /// ```
    /// use provenance::ProvenanceMap;
    /// let mut map = ProvenanceMap::<i32>::new().unwrap();
    ///
    /// let key = map.insert(5);
    /// assert_eq!(&5, map.get(key));
    /// ```
    ///
    /// Multiple equivalent values may be inserted into the map.
    /// Each will get a unique key.
    /// ```
    /// use provenance::ProvenanceMap;
    /// let mut map = ProvenanceMap::<i32>::new().unwrap();
    ///
    /// let key1 = map.insert(5);
    /// let key2 = map.insert(5);
    /// assert_ne!(key1, key2);
    /// ```
    pub fn insert(&mut self, value: Value) -> Key<Value> {
        self.inner.insert(value)
    }

    /// Use a [key](Key) to retrieve an immutable reference to
    /// a stored value.
    /// ```
    /// use provenance::ProvenanceMap;
    /// let mut map = ProvenanceMap::<i32>::new().unwrap();
    ///
    /// let key = map.insert(15);
    /// assert_eq!(&15, map.get(key));
    /// ```
    pub fn get(&self, key: Key<Value>) -> &Value {
        self.inner.get(key)
    }

    /// Use a [key](Key) to retrieve an mutable reference to
    /// a stored value.
    /// ```
    /// use provenance::ProvenanceMap;
    /// let mut map = ProvenanceMap::<i32>::new().unwrap();
    ///
    /// let key = map.insert(15);
    /// assert_eq!(&mut 15, map.get_mut(key));
    /// ```
    pub fn get_mut(&mut self, key: Key<Value>) -> &mut Value {
        self.inner.get_mut(key)
    }

    /// Get an [iterator](Iterator) over all entries in the map.
    /// ```
    /// use provenance::ProvenanceMap;
    /// let mut map = ProvenanceMap::<i32>::new().unwrap();
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
    pub fn entries(&self) -> impl Iterator<Item = (Key<Value>, &Value)> {
        self.inner.entries()
    }

    /// Get an [iterator](Iterator) over mutable references to all entries in the map.
    /// ```
    /// use provenance::ProvenanceMap;
    /// let mut map = ProvenanceMap::<i32>::new().unwrap();
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
    pub fn entries_mut(&mut self) -> impl Iterator<Item = (Key<Value>, &mut Value)> {
        self.inner.entries_mut()
    }

    /// Get an [iterator](Iterator) over all keys in the map.
    /// ```
    /// use provenance::ProvenanceMap;
    /// let mut map = ProvenanceMap::<i32>::new().unwrap();
    ///
    /// map.insert(1);
    /// map.insert(2);
    /// map.insert(3);
    ///
    /// assert_eq!(3, map.keys().count());
    /// ```
    pub fn keys(&self) -> impl Iterator<Item = Key<Value>> {
        self.inner.keys()
    }

    /// Get an [iterator](Iterator) over immutable references to each value in the map.
    /// ```
    /// use provenance::ProvenanceMap;
    /// let mut map = ProvenanceMap::<i32>::new().unwrap();
    ///
    /// map.insert(1);
    /// map.insert(2);
    /// map.insert(3);
    ///
    /// assert_eq!(6, map.iter().sum());
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &Value> {
        self.inner.iter()
    }

    /// Get an [iterator](Iterator) over mutable references to each value in the map.
    /// ```
    /// use provenance::ProvenanceMap;
    /// let mut map = ProvenanceMap::<i32>::new().unwrap();
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
        self.inner.iter_mut()
    }

    /// Search the map in insertion order for the first value that satisfy the given predicate.
    /// If such value is found, an immutable reference to it is returned,
    /// ```
    /// use provenance::ProvenanceMap;
    /// let mut map = ProvenanceMap::<i32>::new().unwrap();
    ///
    /// map.insert(1);
    /// map.insert(2);
    /// map.insert(3);
    ///
    /// assert_eq!(Some(&2), map.find(|&val| val == 2));
    /// ```
    /// otherwise [`None`](std::option::Option::None) is returned.
    /// ```
    /// use provenance::ProvenanceMap;
    /// let mut map = ProvenanceMap::<i32>::new().unwrap();
    ///
    /// map.insert(1);
    /// map.insert(2);
    /// map.insert(3);
    ///
    /// assert_eq!(None, map.find(|&val| val == 53));
    /// ```
    pub fn find<P: Fn(&Value) -> bool>(&self, predicate: P) -> Option<&Value> {
        self.inner.find(predicate)
    }

    /// Search the map in insertion order for the first value that satisfy the given predicate.
    /// If such value is found, a mutable reference to it is returned,
    /// ```
    /// use provenance::ProvenanceMap;
    /// let mut map = ProvenanceMap::<i32>::new().unwrap();
    ///
    /// map.insert(1);
    /// map.insert(2);
    /// map.insert(3);
    ///
    /// assert_eq!(Some(&mut 2), map.find_mut(|&val| val == 2));
    /// ```
    /// otherwise [`None`](std::option::Option::None) is returned.
    /// ```
    /// use provenance::ProvenanceMap;
    /// let mut map = ProvenanceMap::<i32>::new().unwrap();
    ///
    /// map.insert(1);
    /// map.insert(2);
    /// map.insert(3);
    ///
    /// assert_eq!(None, map.find_mut(|&val| val == 53));
    /// ```
    pub fn find_mut<P: Fn(&Value) -> bool>(&mut self, predicate: P) -> Option<&mut Value> {
        self.inner.find_mut(predicate)
    }

    /// Start a transform clone of the map.
    ///
    /// During a transform, all elements of the map are mapped to a new type.
    /// Keys can also be mapped over to the corresponding key in the new map.
    ///
    /// This enables initialisation of complex self-referential structures.
    ///
    /// ```
    /// use std::convert::Infallible;
    /// use provenance::{ProvenanceMap, Key};
    ///
    /// #[derive(Debug)]
    /// struct InitializationNode {
    ///     name: &'static str,
    ///     previous: Option<Key<InitializationNode>>
    /// }
    ///
    /// #[derive(Debug)]
    /// struct Node {
    ///     name: &'static str,
    ///     previous: Key<Node>,
    /// }
    ///
    /// let mut init_map = ProvenanceMap::new().unwrap();
    ///
    /// let first_init_node_key = init_map.insert(InitializationNode { name: "first", previous: None });
    /// let second_init_node_key = init_map.insert(InitializationNode { name: "second", previous: Some(first_init_node_key) });
    ///
    /// let (map, [first_node_key]) = init_map
    ///     .transform()
    ///     .with_references([first_init_node_key])
    ///     .with_transform(|element, key_mapper| {
    ///         // only the first node had no previous set, set it to the last key
    ///         // to create a complete circle of previous references
    ///         let old_key = element.previous.unwrap_or(second_init_node_key);
    ///         let new_key = key_mapper(old_key);
    ///         let new_node = Node { name: element.name, previous: new_key };
    ///         Ok::<_, Infallible>(new_node) // transform is allowed to fail, but we don't need it here
    ///     }).unwrap();
    ///
    /// let first_node = map.get(first_node_key);
    /// assert_eq!(first_node.name, "first");
    ///
    /// let second_node = map.get(first_node.previous);
    /// assert_eq!(second_node.name, "second");
    ///
    /// let first_node_again = map.get(second_node.previous);
    /// assert_eq!(first_node_again.name, "first");
    /// ```
    ///
    /// Just like when creating a new stand alone map, the provenance must be unique.
    pub fn transform(&self) -> ProvenanceMapTransformer<'_, Value> {
        ProvenanceMapTransformer::new_from(self)
    }
}