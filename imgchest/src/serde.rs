pub(crate) mod u8_to_bool {
    use serde::de::Error;
    use serde::de::Unexpected;

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: u8 = serde::Deserialize::deserialize(deserializer)?;
        match value {
            0 => Ok(false),
            1 => Ok(true),
            n => Err(D::Error::invalid_type(
                Unexpected::Unsigned(n.into()),
                &"an integer that is either 0 or 1",
            )),
        }
    }

    pub(crate) fn serialize<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(u8::from(*value))
    }
}

pub(crate) mod from_str_to_str {
    use serde::de::Error;
    use std::borrow::Cow;

    pub(crate) fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display,
    {
        let value: Cow<str> = serde::Deserialize::deserialize(deserializer)?;
        let value: T = value.parse().map_err(D::Error::custom)?;
        Ok(value)
    }

    pub(crate) fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: std::string::ToString,
    {
        let value = value.to_string();
        serializer.serialize_str(&value)
    }
}
