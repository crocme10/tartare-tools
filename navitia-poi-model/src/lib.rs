#![doc(html_root_url = "https://docs.rs/navitia-poi-model/0.1.0")]
#![deny(missing_docs, warnings, missing_debug_implementations)]
//
// Copyright 2017 Kisio Digital and/or its affiliates.
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see
// <http://www.gnu.org/licenses/>.

//! Data structures and functions to manipulate Points of Interest (POIs)

mod io;
pub mod objects;

pub use objects::*;

/// The data type for errors in [navitia-poi-model], just an alias
pub type Error = failure::Error;

/// The classic alias for result type.
pub type Result<T> = std::result::Result<T, Error>;
