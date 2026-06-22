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
        T: std::fmt::Display,
    {
        serializer.collect_str(&value)
    }
}

pub(crate) mod iso8601_string {
    use jiff::Zoned;
    use jiff::fmt::temporal::Pieces;
    use jiff::fmt::temporal::PiecesOffset;
    use jiff::tz::Offset;
    use jiff::tz::TimeZone;

    #[derive(thiserror::Error, Debug)]
    enum Iso8601PiecesToZonedError {
        #[error(transparent)]
        Jiff(#[from] jiff::Error),

        #[error("datetime string missing offset")]
        MissingOffset,
    }

    fn iso8601_str_to_zoned(value: &str) -> Result<Zoned, Iso8601PiecesToZonedError> {
        let pieces = Pieces::parse(value)?;
        let time = pieces.time().unwrap_or_else(jiff::civil::Time::midnight);
        let datetime = pieces.date().to_datetime(time);
        let Some(offset) = pieces.to_numeric_offset() else {
            return Err(Iso8601PiecesToZonedError::MissingOffset);
        };
        let zoned = TimeZone::fixed(offset).to_zoned(datetime)?;

        Ok(zoned)
    }

    fn zoned_to_iso8601_pieces(value: &Zoned) -> Pieces<'_> {
        let timestamp = value.timestamp();
        let offset = Offset::UTC;
        let mut pieces = Pieces::from((timestamp, offset));
        if pieces
            .offset()
            .is_some_and(|offset| offset.to_numeric_offset() == Offset::UTC)
        {
            pieces = pieces.with_offset(PiecesOffset::Zulu);
        }
        pieces
    }

    pub(crate) fn deserialize<'a, D>(deserializer: D) -> Result<Zoned, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = Zoned;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an iso 8601 datetime string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                iso8601_str_to_zoned(value).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(Visitor)
    }

    pub(crate) fn serialize<S>(value: &Zoned, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let pieces = zoned_to_iso8601_pieces(value);
        serializer.collect_str(&pieces)
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_zoned_to_iso8601_pieces() {
            let original = "2019-11-03T00:36:00.000000Z";
            let zoned = iso8601_str_to_zoned(original).unwrap();
            let pieces = zoned_to_iso8601_pieces(&zoned);
            let serialized = format!("{pieces:.6}");
            assert!(serialized == original, "{serialized} != {original}");
        }
    }
}

pub(crate) mod mdy_date {
    use jiff::civil::Date;
    use serde::de::Error;
    use std::borrow::Cow;

    const FORMAT: &str = "%m/%d/%Y";

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Date, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: Cow<str> = serde::Deserialize::deserialize(deserializer)?;
        let value = Date::strptime(FORMAT, &*value).map_err(D::Error::custom)?;
        Ok(value)
    }

    pub(crate) fn serialize<S>(value: &Date, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&value.strftime(FORMAT))
    }
}
