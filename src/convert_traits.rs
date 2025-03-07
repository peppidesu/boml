use std::collections::HashMap;

use crate::{
	table::TomlGetError,
	types::{TomlValue, TomlValueType},
};

/// Error type returned by `FromToml::from_toml`.
#[derive(Debug)]
pub enum FromTomlError<'a, 'b: 'a> {
	/// There was no value to convert.
	Missing,
	/// The key was invalid
	InvalidKey(&'a str),
	/// The value had a different type than expected.
	TypeMismatch(&'a TomlValue<'a, 'b>, TomlValueType),
}

impl<'a, 'b: 'a> FromTomlError<'a, 'b> {
	/// Converts to `InvalidKey` if the error variant is `Missing`.
	pub fn add_key_context(self, key: &'a str) -> Self {
		match self {
			FromTomlError::Missing => FromTomlError::InvalidKey(key),
			other => other,
		}
	}
}

impl<'a, 'b: 'a> From<TomlGetError<'a>> for FromTomlError<'a, 'b> {
	fn from(e: TomlGetError<'a>) -> Self {
		match e {
			TomlGetError::InvalidKey => FromTomlError::Missing,
			TomlGetError::TypeMismatch(v, ty) => FromTomlError::TypeMismatch(v, ty),
		}
	}
}

/// A trait for types that can be constructed from a TOML value. Used by the derive macro.
///
/// This trait is implemented for all types that implement `TryFrom<&'a TomlValue<'a>, Error = ()>`.
pub trait FromToml<'a, 'b: 'a>: Sized {
	/// Constructs a value from a TOML value.
	fn from_toml(value: Option<&'a TomlValue<'a, 'b>>) -> Result<Self, FromTomlError<'a, 'b>>;
}

impl<'a, 'b: 'a, T> FromToml<'a, 'b> for T
where
	T: TryFrom<&'a TomlValue<'a, 'a>, Error = ()>,
{
	fn from_toml(value: Option<&'a TomlValue<'a, 'a>>) -> Result<Self, FromTomlError<'a, 'b>> {
		match value {
			Some(v) => T::try_from(v).map_err(|_| FromTomlError::TypeMismatch(v, v.ty())),
			None => Err(FromTomlError::Missing),
		}
	}
}

impl<'a, 'b: 'a, T> FromToml<'a, 'b>for Vec<T>
where
	T: FromToml<'a, 'b>,
{
	fn from_toml(value: Option<&'a TomlValue<'a, 'b>>) -> Result<Self, FromTomlError<'a, 'b>> {
		match value {
			Some(TomlValue::Array(arr, _)) => arr.iter().map(|v| T::from_toml(Some(v))).collect(),
			Some(v) => Err(FromTomlError::TypeMismatch(v, TomlValueType::Array)),
			None => Err(FromTomlError::Missing),
		}
	}
}

impl<'a, 'b: 'a, T> FromToml<'a, 'b> for Option<T>
where
	T: FromToml<'a, 'b>,
{
	fn from_toml(value: Option<&'a TomlValue<'a, 'b>>) -> Result<Self, FromTomlError<'a, 'b>> {
		match value {
			Some(v) => Ok(Some(T::from_toml(Some(v))?)),
			None => Ok(None),
		}
	}
}
impl<'a, 'b: 'a, T> FromToml<'a, 'b> for HashMap<&'a str, T>
where
	T: FromToml<'a, 'b>,
{
	fn from_toml(value: Option<&'a TomlValue<'a, 'b>>) -> Result<Self, FromTomlError<'a, 'b>> {
		match value {
			Some(TomlValue::Table(table)) => table
				.map
				.iter()
				.map(|(k, v)| Ok((k.as_str(), T::from_toml(Some(v))?)))
				.collect(),
			Some(v) => Err(FromTomlError::TypeMismatch(v, TomlValueType::Table)),
			None => Err(FromTomlError::Missing),
		}
	}
}

/// Inverse trait of `FromToml`. Used to convert a TOML value into a type.
pub trait TomlTryInto<'a, 'b: 'a, T>: Sized {
	/// Converts the TOML value into `T``.
	fn toml_try_into(self) -> Result<T, FromTomlError<'a, 'b>>;
}

impl<'a, 'b: 'a, T> TomlTryInto<'a, 'b, T> for Option<&'a TomlValue<'a, 'b>>
where
	T: FromToml<'a, 'b>,
{
	fn toml_try_into(self) -> Result<T, FromTomlError<'a, 'b>> {
		T::from_toml(self)
	}
}
