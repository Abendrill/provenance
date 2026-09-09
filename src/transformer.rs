use crate::{Key, ProvenanceMap, SeparateProvenanceMap};
use mitsein::prelude::Vec1;

pub enum SeparateMapTransformError<Provenance, Error> {
    ProvenanceUsed,
    MappingErrors(Vec1<(Key<Provenance>, Error)>),
}

pub type MapTransformError<Value, Error> = SeparateMapTransformError<Value, Error>;

pub type SeparateMapTransformResult<Provenance, Value, OldProvenance, Error> =
    Result<SeparateProvenanceMap<Provenance, Value>, SeparateMapTransformError<OldProvenance, Error>>;

pub type MapTransformResult<Value, OldValue, Error> = SeparateMapTransformResult<Value, Value, OldValue, Error>;

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

pub type MapAndReferenceTransformResult<Value, OldValue, Error, const REFERENCES: usize> = SeparateMapAndReferenceTransformResult<Value, Value, OldValue, Error, REFERENCES>;

pub struct SeparateProvenanceMapTransformer<'map, Provenance, Value> {
    map: &'map SeparateProvenanceMap<Provenance, Value>,
}

fn transform_key<OldValue, NewValue>(key: Key<OldValue>) -> Key<NewValue> {
    Key::new(key.index)
}

impl<'map, Provenance: 'static, Value> SeparateProvenanceMapTransformer<'map, Provenance, Value> {
    fn new_from(
        map: &'map SeparateProvenanceMap<Provenance, Value>,
    ) -> SeparateProvenanceMapTransformer<'map, Provenance, Value> {
        SeparateProvenanceMapTransformer { map }
    }

    pub fn with_references<const N: usize>(
        self,
        references: [Key<Value>; N],
    ) -> SeparateProvenanceMapAndReferencesTransformer<'map, Provenance, Value, N> {
        SeparateProvenanceMapAndReferencesTransformer {
            inner_transformer: self,
            references,
        }
    }

    pub fn with_transform<NewProvenance: 'static, NewValue, Error>(
        self,
        mut transform: impl FnMut(&Value, fn(Key<Value>) -> Key<NewValue>) -> Result<NewValue, Error>,
    ) -> SeparateMapTransformResult<NewProvenance, NewValue, Provenance, Error> {
        let mut new_map = SeparateProvenanceMap::<NewProvenance, NewValue>::new()
            .ok_or(SeparateMapTransformError::ProvenanceUsed)?;
        let mut errors = Vec::new();

        for (key, value) in self.map.entries() {
            match transform(value, transform_key) {
                Ok(new_value) => {
                    let new_key = new_map.insert(new_value);

                    // if there has been no errors, then the key indicies must match
                    // otherwise the guarantee that transformed references should work
                    // would be broken
                    debug_assert!((!errors.is_empty()) || (key.index == new_key.index));
                }
                Err(error) => {
                    errors.push((key, error));
                }
            }
        }

        if let Some(errors) = Vec1::try_from(errors).ok() {
            return Err(SeparateMapTransformError::MappingErrors(errors));
        }

        Ok(new_map)
    }
}

pub struct SeparateProvenanceMapAndReferencesTransformer<
    'map,
    Provenance,
    Value,
    const REFERENCES: usize,
> {
    inner_transformer: SeparateProvenanceMapTransformer<'map, Provenance, Value>,
    references: [Key<Value>; REFERENCES],
}

impl<'map, Provenance: 'static, Value, const REFERENCES: usize>
    SeparateProvenanceMapAndReferencesTransformer<'map, Provenance, Value, REFERENCES>
{
    pub fn with_transform<NewProvenance: 'static, NewValue, Error>(
        self,
        transform: impl FnMut(&Value, fn(Key<Value>) -> Key<NewValue>) -> Result<NewValue, Error>,
    ) -> SeparateMapAndReferenceTransformResult<NewProvenance, NewValue, Provenance, Error, REFERENCES> {
        let new_map = self.inner_transformer.with_transform(transform)?;
        let new_references = self.references.map(transform_key);

        Ok((new_map, new_references))
    }
}

impl<Provenance: 'static, Value> SeparateProvenanceMap<Provenance, Value> {
    pub fn transform(&self) -> SeparateProvenanceMapTransformer<'_, Provenance, Value> {
        SeparateProvenanceMapTransformer::new_from(self)
    }
}


pub struct ProvenanceMapTransformer<'map, Value> {
    inner: SeparateProvenanceMapTransformer<'map, Value, Value>,
}

impl<'map, Value: 'static> ProvenanceMapTransformer<'map, Value> {
    fn new_from(
        map: &'map ProvenanceMap<Value>,
    ) -> ProvenanceMapTransformer<'map, Value> {
        ProvenanceMapTransformer { inner: SeparateProvenanceMapTransformer::new_from(&map.inner) }
    }

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

    pub fn with_transform<NewValue: 'static, Error>(
        self,
        transform: impl FnMut(&Value, fn(Key<Value>) -> Key<NewValue>) -> Result<NewValue, Error>,
    ) -> MapTransformResult<NewValue, Value, Error> {
        self.inner.with_transform(transform)
    }
}

pub struct ProvenanceAndReferencesMapTransformer<'map, Value, const REFERENCES: usize> {
    inner: SeparateProvenanceMapAndReferencesTransformer<'map, Value, Value, REFERENCES>,
}

impl<'map, Value: 'static, const REFERENCES: usize>
ProvenanceAndReferencesMapTransformer<'map, Value, REFERENCES>
{
    pub fn with_transform<NewValue: 'static, Error>(
        self,
        transform: impl FnMut(&Value, fn(Key<Value>) -> Key<NewValue>) -> Result<NewValue, Error>,
    ) -> MapAndReferenceTransformResult<NewValue, Value, Error, REFERENCES> {
        self.inner.with_transform(transform)
    }
}

impl<Value: 'static> ProvenanceMap<Value> {
    pub fn transform(&self) -> ProvenanceMapTransformer<'_, Value> {
        ProvenanceMapTransformer::new_from(self)
    }
}
