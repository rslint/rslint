//! Deserialization of rules objects.

use heck::{CamelCase, KebabCase};
use rslint_core::{get_rule_by_name, get_rule_suggestion, CstRule};
use serde::{
    de::{
        value::MapAccessDeserializer, DeserializeSeed, Error, IntoDeserializer, MapAccess, Visitor,
    },
    Deserialize, Deserializer,
};
use std::{fmt, marker::PhantomData};

pub(crate) fn from_rule_objects<'de, D>(deserializer: D) -> Result<Vec<Box<dyn CstRule>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct TypetagObjects<T> {
        _type: PhantomData<T>,
    }

    impl<'de> Visitor<'de> for TypetagObjects<Box<dyn CstRule>> {
        type Value = Vec<Box<dyn CstRule>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("zero or more rule-to-config pairs")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(key) = map.next_key::<String>()? {
                let de = MapAccessDeserializer::new(Entry {
                    key: Some(key.to_camel_case().into_deserializer()),
                    value: &mut map,
                });
                if get_rule_by_name(&key.to_kebab_case()).is_none() {
                    if let Some(suggestion) = get_rule_suggestion(&key.to_kebab_case()) {
                        return Err(M::Error::custom(format!(
                            "Unknown rule '{}'. did you mean '{}'?",
                            key, suggestion
                        )));
                    } else {
                        return Err(M::Error::custom(format!("Unknown rule '{}'", key)));
                    }
                } else {
                    vec.push(Box::<dyn CstRule>::deserialize(de)?);
                }
            }
            Ok(vec)
        }
    }

    struct Entry<K, V> {
        key: Option<K>,
        value: V,
    }

    impl<'de, K, V> MapAccess<'de> for Entry<K, V>
    where
        K: Deserializer<'de, Error = V::Error>,
        V: MapAccess<'de>,
    {
        type Error = V::Error;

        fn next_key_seed<S>(&mut self, seed: S) -> Result<Option<S::Value>, Self::Error>
        where
            S: DeserializeSeed<'de>,
        {
            self.key.take().map(|key| seed.deserialize(key)).transpose()
        }

        fn next_value_seed<S>(&mut self, seed: S) -> Result<S::Value, Self::Error>
        where
            S: DeserializeSeed<'de>,
        {
            self.value.next_value_seed(seed)
        }
    }

    deserializer.deserialize_map(TypetagObjects { _type: PhantomData })
}
