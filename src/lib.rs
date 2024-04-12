//! `serde_json` is great, but the formatting of the output... Not so much.
//! You get to choose between no whitespace whatsoever (good for interchange, but not human-readable),
//! or spaced-out with newlines between every single element (bad for interchange, and only barely human-readable).
//!
//! This crate provides a middle ground: the overarching structure is formatted like `PrettyFormatter`,
//! but lists and objects consisting entirely of primitive values are formatted on a single line (but still not as densely as `CompactFormatter`).
//! The result looks something like this:
//!
//! ```json
//! {
//!  "INFO": {
//!    "name": "tremble_r1",
//!    "transform": [ 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -0.14999999, -0.099999994, -0.03, 1.0 ],
//!    "flags": [ 1074823168, 1081459343, 0 ]
//!  },
//!  "RTY2": { "material_variant": 0 },
//!  "LIG3": {
//!    "a": [ 1.0, 1.0, 1.0, 1.0 ],
//!    "b": 224,
//!    "c": 1.0,
//!    "d": [ 0.0, 45.0, 0.0, 0.0 ]
//!  },
//!  "INFZ": { "a": 11751, "b": 16629, "c": 11393, "d": 32769 },
//!  "JNTV": {
//!    "a": [ 0.0, 0.0, 0.0 ],
//!    "b": 2
//!  },
//!  "KAN7": "tremble_r1.KAN7",
//!  "PLU3": "tremble_r1.PLU3",
//!  "BBOX": {
//!    "min": [ -1.0, -1.0, 0.0 ],
//!    "max": [ 1.0, 1.0, 2.0 ],
//!    "radius": 0.0
//!  }
//!}
//! ```
//!
//! The space savings varies depending on the data, but some tests achieve around 70% reduction in lines.

use serde::Serialize;
use serde_json::{ser::CompactFormatter as CF, Serializer};
use std::io::Write;

type Result<T = (), E = std::io::Error> = std::result::Result<T, E>;

enum Either<A, B> {
	A(A),
	B(B),
}

impl<A: Write, B: Write> Write for Either<A, B> {
	fn write(&mut self, buf: &[u8]) -> Result<usize> {
		match self {
			Either::A(w) => w.write(buf),
			Either::B(w) => w.write(buf),
		}
	}

	fn flush(&mut self) -> Result {
		match self {
			Either::A(w) => w.flush(),
			Either::B(w) => w.flush(),
		}
	}
}

/// A pretty-printer that saves vertical space on lists of primitive values.
///
/// See module-level documentation for more information.
#[derive(Clone, Debug)]
pub struct Formatter<'a> {
	current_indent: usize,
	buffer: Option<Vec<Vec<u8>>>,
	indent: &'a [u8],
}

impl<'a> Formatter<'a> {
	/// Construct a pretty-printer formatter that defaults to using two spaces for indentation.
	pub fn new() -> Self {
		Formatter::with_indent(b"  ")
	}

	/// Construct a pretty-printer formatter that uses the `indent` string for indentation.
	pub fn with_indent(indent: &'a [u8]) -> Self {
		Formatter {
			current_indent: 0,
			buffer: None,
			indent,
		}
	}
}

impl Default for Formatter<'_> {
	fn default() -> Self {
		Formatter::new()
	}
}

impl Formatter<'_> {
	fn writer<'a, 'b, W: Write + ?Sized>(
		&'a mut self,
		w: &'b mut W,
	) -> Either<&'a mut Vec<u8>, &'b mut W> {
		if let Some(buf) = &mut self.buffer {
			Either::A(buf.last_mut().unwrap())
		} else {
			Either::B(w)
		}
	}

	fn begin<W: Write + ?Sized>(&mut self, w: &mut W, bytes: &[u8]) -> Result {
		self.writer(w).write_all(bytes)?;
		if let Some(buf) = self.buffer.replace(Vec::new()) {
			let mut first = Some(());
			for val in buf {
				if first.take().is_none() {
					w.write_all(b",")?;
				}
				self.indent(w)?;
				w.write_all(&val)?;
			}
		}
		self.current_indent += 1;
		Ok(())
	}

	fn end<W: Write + ?Sized>(&mut self, w: &mut W, bytes: &[u8]) -> Result {
		self.current_indent -= 1;
		if let Some(buf) = self.buffer.take() {
			if !buf.is_empty() {
				let mut first = Some(());
				for val in buf {
					if first.take().is_some() {
						w.write_all(b" ")?;
					} else {
						w.write_all(b", ")?;
					}
					w.write_all(&val)?;
				}
				w.write_all(b" ")?;
			}
		} else {
			self.indent(w)?;
		}
		w.write_all(bytes)?;
		if self.current_indent == 0 {
			self.indent(w)?;
		}
		Ok(())
	}

	fn value<W: Write + ?Sized>(&mut self, w: &mut W, first: bool) -> Result {
		if let Some(buf) = &mut self.buffer {
			buf.push(Vec::new())
		} else if !first {
			w.write_all(b",")?;
			self.indent(w)?;
		}
		Ok(())
	}

	fn indent<W: Write + ?Sized>(&mut self, w: &mut W) -> Result {
		w.write_all(b"\n")?;
		for _ in 0..self.current_indent {
			w.write_all(self.indent)?;
		}
		Ok(())
	}
}

macro_rules! impl_write {
	($name:ident, $ty:ty) => {
		fn $name<W: Write + ?Sized>(&mut self, w: &mut W, value: $ty) -> Result {
			CF.$name(&mut self.writer(w), value)
		}
	};
	($name:ident) => {
		fn $name<W: Write + ?Sized>(&mut self, w: &mut W) -> Result {
			CF.$name(&mut self.writer(w))
		}
	};
}

impl serde_json::ser::Formatter for Formatter<'_> {
	impl_write!(write_null);
	impl_write!(write_bool, bool);
	impl_write!(write_i8, i8);
	impl_write!(write_i16, i16);
	impl_write!(write_i32, i32);
	impl_write!(write_i64, i64);
	impl_write!(write_i128, i128);
	impl_write!(write_u8, u8);
	impl_write!(write_u16, u16);
	impl_write!(write_u32, u32);
	impl_write!(write_u64, u64);
	impl_write!(write_u128, u128);
	impl_write!(write_f32, f32);
	impl_write!(write_f64, f64);
	impl_write!(write_number_str, &str);
	impl_write!(begin_string);
	impl_write!(end_string);
	impl_write!(write_string_fragment, &str);
	impl_write!(write_char_escape, serde_json::ser::CharEscape);
	impl_write!(write_raw_fragment, &str);

	fn begin_array<W: Write + ?Sized>(&mut self, w: &mut W) -> Result {
		self.begin(w, b"[")
	}

	fn end_array<W: Write + ?Sized>(&mut self, w: &mut W) -> Result {
		self.end(w, b"]")
	}

	fn begin_array_value<W: Write + ?Sized>(&mut self, w: &mut W, first: bool) -> Result {
		self.value(w, first)
	}

	fn begin_object<W: Write + ?Sized>(&mut self, w: &mut W) -> Result {
		self.begin(w, b"{")
	}

	fn end_object<W: Write + ?Sized>(&mut self, w: &mut W) -> Result {
		self.end(w, b"}")
	}

	fn begin_object_key<W: Write + ?Sized>(&mut self, w: &mut W, first: bool) -> Result {
		self.value(w, first)
	}

	fn begin_object_value<W: Write + ?Sized>(&mut self, w: &mut W) -> Result {
		self.writer(w).write_all(b": ")
	}
}

/// Serialize the given data structure as pretty-printed JSON into the I/O
/// stream.
///
/// Serialization guarantees it only feeds valid UTF-8 sequences to the writer.
///
/// # Errors
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to
/// fail, or if `T` contains a map with non-string keys.
#[inline]
pub fn to_writer<W: Write, T: Serialize + ?Sized>(
	mut writer: W,
	value: &T,
) -> serde_json::Result<()> {
	let mut ser = Serializer::with_formatter(&mut writer, Formatter::new());
	value.serialize(&mut ser)
}

/// Serialize the given data structure as a pretty-printed JSON byte vector.
///
/// # Errors
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to
/// fail, or if `T` contains a map with non-string keys.
#[inline]
pub fn to_vec<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>> {
	let mut writer = Vec::with_capacity(128);
	to_writer(&mut writer, value)?;
	Ok(writer)
}

/// Serialize the given data structure as a pretty-printed String of JSON.
///
/// # Errors
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to
/// fail, or if `T` contains a map with non-string keys.
#[inline]
pub fn to_string<T: Serialize + ?Sized>(value: &T) -> Result<String> {
	let vec = to_vec(value)?;
	// serde-json uses unsafe here, but I'll take the perf hit
	let string = String::from_utf8(vec).unwrap();
	Ok(string)
}
