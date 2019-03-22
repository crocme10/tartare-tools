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

// we want a custom serialization for coords, and so far the cleanest way
// to do this that has been found is to wrap the coord in another struct

use serde_derive::Deserialize;

#[derive(Debug, Clone)]
pub struct Coord(pub geo::Coordinate<f64>);
impl Coord {
    pub fn new(lon: f64, lat: f64) -> Coord {
        Coord(geo::Coordinate { x: lon, y: lat })
    }
    pub fn lon(&self) -> f64 {
        self.x
    }
    pub fn lat(&self) -> f64 {
        self.y
    }
    pub fn is_default(&self) -> bool {
        self.lat() == 0. && self.lon() == 0.
    }
    pub fn is_valid(&self) -> bool {
        !self.is_default()
            && -90. <= self.lat()
            && self.lat() <= 90.
            && -180. <= self.lon()
            && self.lon() <= 180.
    }
}

impl ::std::ops::Deref for Coord {
    type Target = geo::Coordinate<f64>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Coord {
    fn default() -> Coord {
        Coord(geo::Coordinate { x: 0., y: 0. })
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Property {
    pub key: String,
    pub value: String,
}
#[derive(Debug, Clone)]
pub struct Poi {
    pub id: String,
    pub name: String,
    pub coord: Coord,
    pub poi_type_id: String,
    pub properties: Vec<Property>,
    pub visible: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PoiType {
    pub id: String,
    pub name: String,
}
