use crate::{Key, ProvenanceMap, SeparateProvenanceMap};
use mitsein::prelude::Vec1;
use thiserror::Error;

/// [SeparateMapTransformError](SeparateMapTransformError) represent an error that may occur
/// during a map transformation.
#[derive(Debug, Error)]
pub enum SeparateMapTransformError<Provenance, Error> {
    /// The provenance `Provenance` had already been used.
    #[error("provenance is already used")]
    ProvenanceUsed,

    /// Errors that occurred during the transformation.
    ///
    /// The key identifies the element that the error applies to.
    #[error("{} element(s) could not be transformed", .0.len())]
    MappingErrors(Vec1<(Key<Provenance>, Error)>),
}

/// An alias of [SeparateMapTransformError](SeparateMapTransformError) for the case when mapping
/// a [ProvenanceMap](ProvenanceMap).
pub type MapTransformError<Value, Error> = SeparateMapTransformError<Value, Error>;

/// A [Result] alias for when transforming a [SeparateProvenanceMap].
pub type SeparateMapTransformResult<Provenance, Value, OldProvenance, Error> =
    Result<SeparateProvenanceMap<Provenance, Value>, SeparateMapTransformError<OldProvenance, Error>>;

/// A [Result] alias for when transforming a [ProvenanceMap].
pub type MapTransformResult<Value, OldValue, Error> = SeparateMapTransformResult<Value, Value, OldValue, Error>;

/// A [Result] alias for when transforming a [SeparateProvenanceMap] and references.
pub type SeparateMapAndReferenceTransformResult<
    Provenance,
    Value,
    OldProvenance,
    Error,
    const REFERENCES: usize,
> = Result<
    (
        SeparateProvenanceMap<Provenance, Value>,
        [Key<Provenance>; REFERENCES],
    ),
    SeparateMapTransformError<OldProvenance, Error>,
>;

/// A [Result] alias for when transforming a [ProvenanceMap] and references.
pub type MapAndReferenceTransformResult<Value, OldValue, Error, const REFERENCES: usize> = SeparateMapAndReferenceTransformResult<Value, Value, OldValue, Error, REFERENCES>;

/// A [SeparateProvenanceMapTransformer](SeparateProvenanceMapTransformer) represent a yet to be exectuted map transform.
///
/// It is created through [SeparateProvenanceMap::transform](SeparateProvenanceMap::transform).
///
/// One can either add references to the transform using [with_references](SeparateProvenanceMapTransformer::with_references)
/// or directly execute the transform using [with_transform](SeparateProvenanceMapTransformer::with_transform).
pub struct SeparateProvenanceMapTransformer<'map, Provenance, Value> {
    map: &'map SeparateProvenanceMap<Provenance, Value>,
}

/// Maps a key from one map to another.
///
/// **IMPORTANT**: must only be used when it is guranteed that the
/// same index maps to corresponding elements in the two maps.
fn transform_key<OldValue, NewValue>(key: Key<OldValue>) -> Key<NewValue> {
    Key::new(key.index)
}

impl<'map, Provenance: 'static, Value> SeparateProvenanceMapTransformer<'map, Provenance, Value> {
    /// Convience function to create a new transformer that references a map.
    pub(crate) fn new_from(
        map: &'map SeparateProvenanceMap<Provenance, Value>,
    ) -> SeparateProvenanceMapTransformer<'map, Provenance, Value> {
        SeparateProvenanceMapTransformer { map }
    }

    /// Add references to the transform.
    ///
    /// Useful when one want to keep track of a key (not held by an element) across the transform.
    pub fn with_references<const N: usize>(
        self,
        references: [Key<Provenance>; N],
    ) -> SeparateProvenanceMapAndReferencesTransformer<'map, Provenance, Value, N> {
        SeparateProvenanceMapAndReferencesTransformer {
            inner_transformer: self,
            references,
        }
    }

    /// Execute this transform with the provided closure.
    ///
    /// Note, the closure takes two arguments.
    /// The first argument is the element to transformed.
    /// The second argument is function that the transform can use to map old keys to corresponding new ones.
    ///
    /// The closure may return an error.
    /// If any element transformation result in an error, the entire transformation fails.
    /// All element transformation errors are collected and returned.
    ///
    /// The transformation may also fail if the provenance represented by
    /// the generic parameter `NewProvenance` is already used.
    /// The provenance is only used if the transformation is successful.
    pub fn with_transform<NewProvenance: 'static, NewValue, Error>(
        self,
        mut transform: impl FnMut(&Value, fn(Key<Provenance>) -> Key<NewProvenance>) -> Result<NewValue, Error>,
    ) -> SeparateMapTransformResult<NewProvenance, NewValue, Provenance, Error> {
        let mut new_elements = Vec::new();
        let mut errors = Vec::new();

        for (key, value) in self.map.entries() {
            match transform(value, transform_key) {
                Ok(new_value) => {
                    new_elements.push((key, new_value));
                }
                Err(error) => {
                    errors.push((key, error));
                }
            }
        }

        if let Some(errors) = Vec1::try_from(errors).ok() {
            return Err(SeparateMapTransformError::MappingErrors(errors));
        }

        let mut new_map = SeparateProvenanceMap::<NewProvenance, NewValue>::new()
            .ok_or(SeparateMapTransformError::ProvenanceUsed)?;

        for (old_key, new_element) in new_elements {
            let new_key = new_map.insert(new_element);

            debug_assert_eq!(old_key.index, new_key.index);
        }

        Ok(new_map)
    }
}

/// A [SeparateProvenanceMapAndReferencesTransformer] represent a yet to be exectuted map transform.
///
/// It is created through [SeparateProvenanceMapTransformer::with_references].
///
/// There is only one available operation, i.e. to execute the transform using [with_transform](SeparateProvenanceMapAndReferencesTransformer::with_transform).
pub struct SeparateProvenanceMapAndReferencesTransformer<
    'map,
    Provenance,
    Value,
    const REFERENCES: usize,
> {
    inner_transformer: SeparateProvenanceMapTransformer<'map, Provenance, Value>,
    references: [Key<Provenance>; REFERENCES],
}

impl<'map, Provenance: 'static, Value, const REFERENCES: usize>
    SeparateProvenanceMapAndReferencesTransformer<'map, Provenance, Value, REFERENCES>
{
    /// Execute this transform with the provided closure.
    ///
    /// Note, the closure takes two arguments.
    /// The first argument is the element to transformed.
    /// The second argument is function that the transform can use to map old keys to corresponding new ones.
    ///
    /// The closure may return an error.
    /// If any element transformation result in an error, the entire transformation fails.
    /// All element transformation errors are collected and returned.
    ///
    /// The transformation may also fail if the provenance represented by
    /// the generic parameter `NewProvenance` is already used.
    /// The provenance is only used if the transformation is successful.
    pub fn with_transform<NewProvenance: 'static, NewValue, Error>(
        self,
        transform: impl FnMut(&Value, fn(Key<Provenance>) -> Key<NewProvenance>) -> Result<NewValue, Error>,
    ) -> SeparateMapAndReferenceTransformResult<NewProvenance, NewValue, Provenance, Error, REFERENCES> {
        let new_map = self.inner_transformer.with_transform(transform)?;
        let new_references = self.references.map(transform_key);

        Ok((new_map, new_references))
    }
}

/// A [ProvenanceMapTransformer](ProvenanceMapTransformer) represent a yet to be exectuted map transform.
///
/// It is created through [ProvenanceMap::transform](ProvenanceMap::transform).
///
/// One can either add references to the transform using [with_references](ProvenanceMapTransformer::with_references)
/// or directly execute the transform using [with_transform](ProvenanceMapTransformer::with_transform).
pub struct ProvenanceMapTransformer<'map, Value> {
    inner: SeparateProvenanceMapTransformer<'map, Value, Value>,
}

impl<'map, Value: 'static> ProvenanceMapTransformer<'map, Value> {
    /// Convience function to create a new transformer that references a map.
    pub(crate) fn new_from(
        map: &'map ProvenanceMap<Value>,
    ) -> ProvenanceMapTransformer<'map, Value> {
        ProvenanceMapTransformer { inner: SeparateProvenanceMapTransformer::new_from(&map.inner) }
    }

    /// Add references to the transform.
    ///
    /// Useful when one want to keep track of a key (not held by an element) across the transform.
    pub fn with_references<const N: usize>(
        self,
        references: [Key<Value>; N],
    ) -> ProvenanceAndReferencesMapTransformer<'map, Value, N> {
        ProvenanceAndReferencesMapTransformer {
            inner: SeparateProvenanceMapAndReferencesTransformer {
                inner_transformer: self.inner,
                references,
            }
        }
    }

    /// Execute this transform with the provided closure.
    ///
    /// Note, the closure takes two arguments.
    /// The first argument is the element to transformed.
    /// The second argument is function that the transform can use to map old keys to corresponding new ones.
    ///
    /// The closure may return an error.
    /// If any element transformation result in an error, the entire transformation fails.
    /// All element transformation errors are collected and returned.
    ///
    /// The transformation may also fail if the provenance represented by
    /// the generic parameter `NewValue` is already used.
    /// The provenance is only used if the transformation is successful.
    pub fn with_transform<NewValue: 'static, Error>(
        self,
        transform: impl FnMut(&Value, fn(Key<Value>) -> Key<NewValue>) -> Result<NewValue, Error>,
    ) -> MapTransformResult<NewValue, Value, Error> {
        self.inner.with_transform(transform)
    }
}

/// A [ProvenanceAndReferencesMapTransformer] represent a yet to be exectuted map transform.
///
/// It is created through [ProvenanceMapTransformer::with_references].
///
/// There is only one available operation, i.e. to execute the transform using [with_transform](ProvenanceAndReferencesMapTransformer::with_transform).
pub struct ProvenanceAndReferencesMapTransformer<'map, Value, const REFERENCES: usize> {
    inner: SeparateProvenanceMapAndReferencesTransformer<'map, Value, Value, REFERENCES>,
}

impl<'map, Value: 'static, const REFERENCES: usize>
ProvenanceAndReferencesMapTransformer<'map, Value, REFERENCES>
{
    /// Execute this transform with the provided closure.
    ///
    /// Note, the closure takes two arguments.
    /// The first argument is the element to transformed.
    /// The second argument is function that the transform can use to map old keys to corresponding new ones.
    ///
    /// The closure may return an error.
    /// If any element transformation result in an error, the entire transformation fails.
    /// All element transformation errors are collected and returned.
    ///
    /// The transformation may also fail if the provenance represented by
    /// the generic parameter `NewValue` is already used.
    /// The provenance is only used if the transformation is successful.
    pub fn with_transform<NewValue: 'static, Error>(
        self,
        transform: impl FnMut(&Value, fn(Key<Value>) -> Key<NewValue>) -> Result<NewValue, Error>,
    ) -> MapAndReferenceTransformResult<NewValue, Value, Error, REFERENCES> {
        self.inner.with_transform(transform)
    }
}
