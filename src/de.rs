//! Forgiving decoding of API answers, and the macros that declare models and enums.
//!
//! The API is not always strict about types: numbers arrive as strings, booleans as `1` or `"yes"`, dates with or
//! without an offset, lists as `null`. Every model field is decoded through [`Lenient`], which accepts all of these
//! and falls back to the field's default instead of failing the whole answer.

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::collections::BTreeMap;

/// A type that can be read from any JSON value without failing.
///
/// `None` means "missing or unreadable"; the field then takes its default value.
pub trait Lenient: Sized {
    /// Reads the value, or returns `None` when it is missing or cannot be read.
    fn from_lenient(value: Value) -> Option<Self>;
}

/// `deserialize_with` target used by every model field.
pub(crate) fn lenient<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Lenient + Default,
{
    let value = Value::deserialize(deserializer)?;
    Ok(T::from_lenient(value).unwrap_or_default())
}

impl Lenient for Value {
    fn from_lenient(value: Value) -> Option<Self> {
        Some(value)
    }
}

impl Lenient for String {
    fn from_lenient(value: Value) -> Option<Self> {
        match value {
            Value::Null => None,
            Value::String(s) => Some(s),
            Value::Bool(b) => Some(b.to_string()),
            Value::Number(n) => Some(n.to_string()),
            other => Some(other.to_string()),
        }
    }
}

impl Lenient for bool {
    fn from_lenient(value: Value) -> Option<Self> {
        match value {
            Value::Bool(b) => Some(b),
            Value::Number(n) => n.as_f64().map(|f| f != 0.0),
            Value::String(s) => match s.trim().to_ascii_lowercase().as_str() {
                "true" | "1" | "yes" | "y" | "on" => Some(true),
                "false" | "0" | "no" | "n" | "off" | "" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }
}

fn number(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse::<f64>().ok(),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    }
}

fn integer(value: &Value) -> Option<i64> {
    match value {
        Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)),
        Value::String(s) => {
            let s = s.trim();
            s.parse::<i64>().ok().or_else(|| s.parse::<f64>().ok().map(|f| f as i64))
        }
        Value::Bool(b) => Some(i64::from(*b)),
        _ => None,
    }
}

impl Lenient for f64 {
    fn from_lenient(value: Value) -> Option<Self> {
        number(&value).filter(|f| f.is_finite())
    }
}

macro_rules! lenient_int {
    ($($t:ty),*) => {$(
        impl Lenient for $t {
            fn from_lenient(value: Value) -> Option<Self> {
                integer(&value).and_then(|n| <$t>::try_from(n).ok())
            }
        }
    )*};
}
lenient_int!(i32, i64, u32, u64);

impl<T: Lenient> Lenient for Option<T> {
    fn from_lenient(value: Value) -> Option<Self> {
        Some(T::from_lenient(value))
    }
}

impl<T: Lenient> Lenient for Vec<T> {
    fn from_lenient(value: Value) -> Option<Self> {
        match value {
            Value::Array(items) => Some(items.into_iter().filter_map(T::from_lenient).collect()),
            _ => None,
        }
    }
}

impl Lenient for BTreeMap<String, String> {
    fn from_lenient(value: Value) -> Option<Self> {
        match value {
            Value::Object(map) => {
                Some(map.into_iter().map(|(k, v)| (k, String::from_lenient(v).unwrap_or_default())).collect())
            }
            _ => None,
        }
    }
}

impl Lenient for DateTime<Utc> {
    fn from_lenient(value: Value) -> Option<Self> {
        match value {
            Value::String(s) => parse_time(&s),
            _ => None,
        }
    }
}

/// Parses the timestamps the API sends: RFC 3339, ISO 8601 without an offset (read as UTC), with a space instead of
/// the `T`, or a date alone. `0001-01-01T00:00:00` (.NET's empty date) and unreadable text give `None`.
pub(crate) fn parse_time(text: &str) -> Option<DateTime<Utc>> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    let parsed = DateTime::parse_from_rfc3339(text)
        .map(|t| t.with_timezone(&Utc))
        .ok()
        .or_else(|| DateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S%.f%:z").map(|t| t.with_timezone(&Utc)).ok())
        .or_else(|| {
            ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%dT%H:%M", "%Y-%m-%d %H:%M:%S%.f"]
                .iter()
                .find_map(|format| NaiveDateTime::parse_from_str(text, format).ok())
                .map(|t| t.and_utc())
        })
        .or_else(|| {
            NaiveDate::parse_from_str(text, "%Y-%m-%d").ok().and_then(|d| d.and_hms_opt(0, 0, 0)).map(|t| t.and_utc())
        })?;
    use chrono::Datelike;
    (parsed.year() > 1).then_some(parsed)
}

/// Declares a model: a struct whose fields are read with [`Lenient`] and renamed to the API's JSON names.
///
/// Response models are `#[non_exhaustive]`, so new fields can be added in minor versions. Models that you also build
/// yourself (contacts, DS records) use the `@open` form, which leaves the struct exhaustive for struct literals.
macro_rules! model {
    (
        $(#[$meta:meta])*
        pub struct $name:ident {
            $( $(#[$fmeta:meta])* $json:literal => pub $field:ident : $ty:ty ),* $(,)?
        }
    ) => {
        $crate::de::model! { @define [$(#[$meta])* #[non_exhaustive]] $name { $( $(#[$fmeta])* $json => $field : $ty ),* } }
    };
    (
        @open
        $(#[$meta:meta])*
        pub struct $name:ident {
            $( $(#[$fmeta:meta])* $json:literal => pub $field:ident : $ty:ty ),* $(,)?
        }
    ) => {
        $crate::de::model! { @define [$(#[$meta])*] $name { $( $(#[$fmeta])* $json => $field : $ty ),* } }
    };
    (@define [$($meta:tt)*] $name:ident { $( $(#[$fmeta:meta])* $json:literal => $field:ident : $ty:ty ),* }) => {
        $($meta)*
        #[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
        pub struct $name {
            $(
                $(#[$fmeta])*
                #[serde(rename = $json)]
                pub $field: $ty,
            )*
        }

        const _: () = {
            // Reads only JSON objects (serde would also accept an array of the fields in order), every field leniently.
            #[derive(serde::Deserialize)]
            struct Fields {
                $(
                    #[serde(rename = $json, default, deserialize_with = "crate::de::lenient")]
                    $field: $ty,
                )*
            }

            impl<'de> serde::Deserialize<'de> for $name {
                fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                    let value = serde_json::Value::deserialize(deserializer)?;
                    if !value.is_object() {
                        return Err(<D::Error as serde::de::Error>::custom(concat!(
                            "expected a JSON object for ",
                            stringify!($name)
                        )));
                    }
                    let fields: Fields = serde_json::from_value(value).map_err(<D::Error as serde::de::Error>::custom)?;
                    Ok($name { $( $field: fields.$field ),* })
                }
            }
        };

        impl $crate::de::Lenient for $name {
            fn from_lenient(value: serde_json::Value) -> Option<Self> {
                if value.is_object() {
                    serde_json::from_value(value).ok()
                } else {
                    None
                }
            }
        }
    };
}
pub(crate) use model;

/// Declares an enum the API sends as a number. Unknown numbers become `Unknown(n)`; a missing or unreadable value is
/// `Unknown(-1)`.
macro_rules! int_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $( $(#[$vmeta:meta])* $variant:ident = $code:literal => $wire:literal ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        pub enum $name {
            $( $(#[$vmeta])* $variant, )*
            /// A value this version of the SDK does not know: the number the API sent, or `-1` when it sent no
            /// readable value.
            Unknown(i64),
        }

        impl $name {
            /// Every known value, in the API's order.
            pub const ALL: &'static [$name] = &[$($name::$variant),*];

            /// Returns the number the API uses for this value.
            pub fn code(self) -> i64 {
                match self {
                    $( $name::$variant => $code, )*
                    $name::Unknown(code) => code,
                }
            }

            /// Returns the value for the API's number; unknown numbers give `Unknown(code)`.
            pub fn from_code(code: i64) -> Self {
                match code {
                    $( $code => $name::$variant, )*
                    other => $name::Unknown(other),
                }
            }

            /// Returns the API's name for this value, for example `"Active"`, or `"Unknown"`.
            pub fn name(self) -> &'static str {
                match self {
                    $( $name::$variant => $wire, )*
                    $name::Unknown(_) => "Unknown",
                }
            }

            /// Returns whether this is a value this version of the SDK knows.
            pub fn is_known(self) -> bool {
                !matches!(self, $name::Unknown(_))
            }
        }

        impl Default for $name {
            /// `Unknown(-1)`: no value.
            fn default() -> Self {
                $name::Unknown(-1)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $name::Unknown(code) => write!(f, "Unknown({code})"),
                    other => f.write_str(other.name()),
                }
            }
        }

        impl std::str::FromStr for $name {
            type Err = $crate::Error;

            /// Accepts the name (any case) or the number of a known value.
            fn from_str(text: &str) -> Result<Self, Self::Err> {
                let text = text.trim();
                if let Some(value) = $name::ALL.iter().find(|v| v.name().eq_ignore_ascii_case(text)) {
                    return Ok(*value);
                }
                if let Ok(code) = text.parse::<i64>() {
                    let value = $name::from_code(code);
                    if value.is_known() {
                        return Ok(value);
                    }
                }
                let names: Vec<&str> = $name::ALL.iter().map(|v| v.name()).collect();
                Err($crate::Error::invalid_input(
                    stringify!($name),
                    format!("'{}' is not a valid {} (allowed: {})", text, stringify!($name), names.join(", ")),
                ))
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_i64(self.code())
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = serde_json::Value::deserialize(deserializer)?;
                Ok(<$name as $crate::de::Lenient>::from_lenient(value).unwrap_or_default())
            }
        }

        impl $crate::de::Lenient for $name {
            fn from_lenient(value: serde_json::Value) -> Option<Self> {
                match value {
                    serde_json::Value::Number(n) => n.as_i64().map($name::from_code),
                    serde_json::Value::String(s) => Some(s.parse::<$name>().unwrap_or_else(|_| {
                        s.trim().parse::<i64>().map($name::from_code).unwrap_or($name::Unknown(-1))
                    })),
                    _ => None,
                }
            }
        }
    };
}
pub(crate) use int_enum;

/// Declares an enum the API sends as text. Values the SDK does not know are kept as `Other(text)`.
macro_rules! str_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $( $(#[$vmeta:meta])* $variant:ident => $wire:literal $([$($alias:literal),*])? ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        pub enum $name {
            $( $(#[$vmeta])* $variant, )*
            /// A value this version of the SDK does not know, as the API sent it.
            Other(String),
        }

        impl $name {
            /// Every known value.
            pub const ALL: &'static [$name] = &[$($name::$variant),*];

            /// Returns the text the API uses for this value.
            pub fn as_str(&self) -> &str {
                match self {
                    $( $name::$variant => $wire, )*
                    $name::Other(text) => text.as_str(),
                }
            }

            /// Reads a value, ignoring case and surrounding spaces. Unknown text gives `Other(text)`.
            pub fn parse(text: &str) -> Self {
                let text = text.trim();
                $(
                    if text.eq_ignore_ascii_case($wire) $($(|| text.eq_ignore_ascii_case($alias))*)? {
                        return $name::$variant;
                    }
                )*
                $name::Other(text.to_string())
            }

            /// Returns whether this is a value this version of the SDK knows.
            pub fn is_known(&self) -> bool {
                !matches!(self, $name::Other(_))
            }
        }

        impl Default for $name {
            /// `Other("")`: no value.
            fn default() -> Self {
                $name::Other(String::new())
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl From<&str> for $name {
            fn from(text: &str) -> Self {
                $name::parse(text)
            }
        }

        impl From<String> for $name {
            fn from(text: String) -> Self {
                $name::parse(&text)
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = serde_json::Value::deserialize(deserializer)?;
                Ok(<$name as $crate::de::Lenient>::from_lenient(value).unwrap_or_default())
            }
        }

        impl $crate::de::Lenient for $name {
            fn from_lenient(value: serde_json::Value) -> Option<Self> {
                String::from_lenient(value).map(|s| $name::parse(&s))
            }
        }
    };
}
pub(crate) use str_enum;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn strings_accept_numbers_and_booleans() {
        assert_eq!(String::from_lenient(json!(12)), Some("12".to_string()));
        assert_eq!(String::from_lenient(json!(true)), Some("true".to_string()));
        assert_eq!(String::from_lenient(json!(null)), None);
    }

    #[test]
    fn numbers_accept_strings() {
        assert_eq!(i64::from_lenient(json!("42")), Some(42));
        assert_eq!(i64::from_lenient(json!("42.0")), Some(42));
        assert_eq!(i32::from_lenient(json!(7.9)), Some(7));
        assert_eq!(f64::from_lenient(json!("11.99")), Some(11.99));
        assert_eq!(u32::from_lenient(json!(-1)), None);
        assert_eq!(i64::from_lenient(json!("abc")), None);
    }

    #[test]
    fn booleans_accept_flags() {
        for yes in [json!(true), json!(1), json!("1"), json!("yes"), json!("True"), json!("on")] {
            assert_eq!(bool::from_lenient(yes.clone()), Some(true), "{yes}");
        }
        for no in [json!(false), json!(0), json!("0"), json!("no"), json!("")] {
            assert_eq!(bool::from_lenient(no.clone()), Some(false), "{no}");
        }
        assert_eq!(bool::from_lenient(json!("maybe")), None);
    }

    #[test]
    fn lists_skip_null_and_bad_items() {
        assert_eq!(Vec::<i64>::from_lenient(json!([1, "2", "x", null])), Some(vec![1, 2]));
        assert_eq!(Vec::<i64>::from_lenient(json!(null)), None);
    }

    #[test]
    fn attributes_turn_values_into_text() {
        let map = BTreeMap::<String, String>::from_lenient(json!({"a": 1, "b": true, "c": null, "d": "x"})).unwrap();
        assert_eq!(map["a"], "1");
        assert_eq!(map["b"], "true");
        assert_eq!(map["c"], "");
        assert_eq!(map["d"], "x");
        assert_eq!(BTreeMap::<String, String>::from_lenient(json!([])), None);
    }

    #[test]
    fn times_with_and_without_offset() {
        let utc = parse_time("2026-03-15T10:20:30Z").unwrap();
        assert_eq!(parse_time("2026-03-15T10:20:30").unwrap(), utc);
        assert_eq!(parse_time("2026-03-15T13:20:30+03:00").unwrap(), utc);
        assert_eq!(parse_time("2026-03-15 10:20:30").unwrap(), utc);
        assert_eq!(parse_time("2026-03-15T10:20:30.1234567").unwrap().timestamp(), utc.timestamp());
        assert_eq!(parse_time("2026-03-15").unwrap().to_rfc3339(), "2026-03-15T00:00:00+00:00");
        assert_eq!(parse_time("0001-01-01T00:00:00"), None);
        assert_eq!(parse_time("not a date"), None);
        assert_eq!(parse_time(""), None);
    }
}
