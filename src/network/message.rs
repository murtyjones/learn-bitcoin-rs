//! Network message
//!
//! this module defines the `Message` traits whciha re used
//! for (de)serializing Bitcoin objects for transmissin on
//! the netowrk. It also defines (de)serialization routes for
//! many primitives.

use std::borrow::Cow;

pub struct CommandString(Cow<'static, str>);
