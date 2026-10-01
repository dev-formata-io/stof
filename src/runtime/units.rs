//
// Copyright 2025 Formata, Inc. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use std::f64::consts::PI;
use arcstr::{literal, ArcStr};
use serde::{Deserialize, Serialize};


/// Units.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub enum Units {
    /// No units
    #[default]
    None,

    /// Undefined units.
    Undefined,

    /// Metric length units.
    Kilometers, // km 1000m
    Hectometers,// hm 100m
    Decameters, // dcm 10m
    Meters,     // m 1m
    Decimeters, // dm 1/10m
    Centimeters,// cm 1/100m
    Millimeters,// mm 1/1000m
    Micrometers,// um 1/1000000m
    Nanometers, // nm 1/1000000000m
    
    // Imperial length units.
    Miles,      // mi 5280ft
    Yards,      // yd 3ft
    Feet,       // ft 12inches
    Inches,     // inches

    // Time units.
    Days,         // day 24hr
    Hours,        // hr  60min
    Minutes,      // min 60s
    Seconds,      // s
    Milliseconds, // ms 10^-3 s
    Microseconds, // us 10^-6 s
    Nanoseconds,  // ns 10^-9 s

    // Temperature units.
    Kelvin,     // K -273.15 = C
    Celsius,    // C
    Fahrenheit, // F (x - 32)*(5/9) = C

    // Metric mass units.
    Gigatonnes, // Gt 1 000 000 000 000 000 g
    Megatonnes, // Mt 1 000 000 000 000 g
    Tonnes,     // t  1 000 000 g
    Kilograms,  // kg 1 000 g
    Grams,      // g
    Milligrams, // mg 0.001 g
    Micrograms, // ug 0.000 001 g
    Nanograms,  // ng 0.000 000 001 g
    Picograms,  // pg 0.000 000 000 001 g

    // Imperial mass units (US).
    Tons,   // ton 0.907 tonnes
    Pounds, // lb 453.592 g
    Ounce,  // oz 28.3495 g

    // Angles
    Degrees,
    Radians,
    PositiveDegrees, // Degrees, kept positive
    PositiveRadians, // Radians, kept positive

    // Computer Memory
    Bits,
    Bytes,
    Kibibytes,
    Kilobytes,
    Mebibytes,
    Megabytes,
    Gibibytes,
    Gigabytes,
    Tebibytes,
    Terabytes,
    Pebibytes,
    Petabytes,
    Exbibyte,
    Exabytes,
    Zebibytes,
    Zettabytes,
    Yobibytes,
    Yottabytes,
}
impl<T: AsRef<str>> From<T> for Units {
    fn from(value: T) -> Self {
        let value = value.as_ref();
        if value.len() < 1 {
            return Self::None;
        }
        match value {
            // Angles
            "rad" | "radians" => Self::Radians,
            "deg" | "degrees" => Self::Degrees,
            "prad" | "pradians" => Self::PositiveRadians,
            "pdeg" | "pdegrees" => Self::PositiveDegrees,

            // Metric length
            "km" | "kilometers" => Self::Kilometers,
            "hm" | "hectometers" => Self::Hectometers,
            "dcm" | "decameters" => Self::Decameters,
            "m" | "meters" => Self::Meters,
            "dm" | "decimeters" => Self::Decimeters,
            "cm" | "centimeters" => Self::Centimeters,
            "mm" | "millimeters" => Self::Millimeters,
            "um" | "micrometers" => Self::Micrometers,
            "nm" | "nanometers" => Self::Nanometers,

            // Imperial length
            "mi" | "miles" => Self::Miles,
            "yd" | "yards" => Self::Yards,
            "ft" | "feet" => Self::Feet,
            "in" | "inches" => Self::Inches,

            // Time units
            "day" | "days" => Self::Days,
            "hr" | "hours" => Self::Hours,
            "min" | "minutes" => Self::Minutes,
            "s" | "second" | "seconds" => Self::Seconds,
            "ms" | "milliseconds" => Self::Milliseconds,
            "us" | "microseconds" => Self::Microseconds,
            "ns" | "nanoseconds" => Self::Nanoseconds,

            // Temperature units
            "K" | "kelvin" => Self::Kelvin,
            "C" | "celsius" => Self::Celsius,
            "F" | "fahrenheit" => Self::Fahrenheit,

            // Metric mass units
            "Gt" | "gigatonnes" => Self::Gigatonnes,
            "Mt" | "megatonnes" => Self::Megatonnes,
            "t" | "tonnes" => Self::Tonnes,
            "kg" | "kilograms" => Self::Kilograms,
            "g" | "grams" => Self::Grams,
            "mg" | "milligrams" => Self::Milligrams,
            "ug" | "micrograms" => Self::Micrograms,
            "ng" | "nanograms" => Self::Nanograms,
            "pg" | "picograms" => Self::Picograms,

            // Imperial mass units (US)
            "tons" | "Ton" => Self::Tons,
            "lb" | "lbs" => Self::Pounds,
            "oz" | "ounces" => Self::Ounce,

            // Computer memory
            "bit" | "bits" => Self::Bits,
            "byte" | "bytes" => Self::Bytes,
            "KiB" | "kibibytes" => Self::Kibibytes,
            "KB" | "kilobytes" => Self::Kilobytes,
            "MiB" | "mebibytes" => Self::Mebibytes,
            "MB" | "megabytes" => Self::Megabytes,
            "GiB" | "gibibytes" => Self::Gibibytes,
            "GB" | "gigabytes" => Self::Gigabytes,
            "TiB" | "tebibytes" => Self::Tebibytes,
            "TB" | "terabytes" => Self::Terabytes,
            "PiB" | "pebibytes" => Self::Pebibytes,
            "PB" | "petabytes" => Self::Petabytes,
            "EiB" | "exbibytes" => Self::Exbibyte,
            "EB" | "exabytes" => Self::Exabytes,
            "ZiB" | "zebibytes" => Self::Zebibytes,
            "ZB" | "zettabytes" => Self::Zettabytes,
            "YiB" | "yobibytes" => Self::Yobibytes,
            "YB" | "yottabytes" => Self::Yottabytes,
            _ => Self::Undefined,
        }
    }
}
impl Units {
    /// Common unit.
    pub fn common(&self, other: Self) -> Self {
        // No units
        if !self.has_units() { return other; }
        if !other.has_units() { return *self; }

        // Undefined units
        if self.is_undefined() || other.is_undefined() {
            return Self::Undefined;
        }

        // Eq units
        if *self == other {
            return other;
        }

        // Angle units - Base is always Radians if mixed!
        if self.is_angle() && other.is_angle() {
            if self.is_degrees() && other.is_degrees() {
                // One of em has to be a PositiveDegrees... neq above
                return Self::PositiveDegrees;
            }
            // Mixed values and radians get cast to Radians!
            if self.is_positive_angle() || other.is_positive_angle() {
                return Self::PositiveRadians;
            }
            return Self::Radians;
        }

        // Length units
        if self.is_length() && other.is_length() {
            if self.is_metric_length() && other.is_metric_length() {
                // If both metric, take the larger unit as the common unit
                if other < *self {
                    return other;
                }
                return *self;
            }
            if self.is_imperial_length() && other.is_imperial_length() {
                if other < *self {
                    return other;
                }
                return *self;
            }
            // If one is metric and one is imperial, we go with the SI base unit (Meters)
            return Self::Meters;
        } else if self.is_length() || other.is_length() {
            return Self::Undefined;
        }

        // Time units
        if self.is_time() && other.is_time() {
            if other < *self { // take larger time unit
                return other;
            }
            return *self;
        } else if self.is_time() || other.is_time() {
            return Self::Undefined;
        }

        // Tempurature units
        if self.is_temperature() && other.is_temperature() {
            if other < *self { return other; }
            return *self;
        } else if self.is_temperature() || other.is_temperature() {
            return Self::Undefined;
        }

        // Mass units
        if self.is_mass() && other.is_mass() {
            if self.is_metric_mass() && other.is_metric_mass() {
                if other < *self { return other; }
                return *self;
            }
            if self.is_imperial_mass() && other.is_imperial_mass() {
                if other < *self { return other; }
                return *self;
            }
            // If one is imperial and one is metric, we go with the SI base unit (kg)
            return Self::Kilograms;
        } else if self.is_mass() || other.is_mass() {
            return Self::Undefined;
        }

        // Memory units
        if self.is_memory() && other.is_memory() {
            if other > *self { return other; }
            return *self;
        } else if self.is_memory() || other.is_memory() {
            return Self::Undefined;
        }

        Self::None
    }

    /// Float convert between units.
    pub fn convert(value: f64, units: Self, to: Self) -> Result<f64, String> {
        // No conversion if either has no units! (either converting to or from "None")
        if !units.has_units() || !to.has_units() { return Ok(value); }

        // Converting to an undefined unit? (cannot convert from an undefined unit!)
        if to.is_undefined() { return Ok(value); }

        // Angle conversion?
        if units.is_angle() && to.is_angle() {
            let rad = Self::to_radians(value, units);
            return Ok(Self::from_radians(rad, to));
        } else if units.is_angle() || to.is_angle() {
            return Err(format!("Cannot convert {:?} to {:?}", units, to));
        }

        // Ratio units (length, time, mass, memory): one exact rational factor between the two units
        if let (Some((dim, from_num, from_den)), Some((to_dim, to_num, to_den))) = (units.factor(), to.factor()) {
            if dim != to_dim {
                return Err(format!("Cannot convert {:?} to {:?}", units, to));
            }
            return Ok(Self::scale(value, from_num * to_den, from_den * to_num));
        } else if units.factor().is_some() || to.factor().is_some() {
            return Err(format!("Cannot convert {:?} to {:?}", units, to));
        }

        // Temp conversion?
        if units.is_temperature() && to.is_temperature() {
            let celc = Self::to_c(value, units);
            return Ok(Self::from_c(celc, to));
        } else if units.is_temperature() || to.is_temperature() {
            return Err(format!("Cannot convert {:?} to {:?}", units, to));
        }

        return Err(format!("Cannot convert {:?} to {:?}", units, to));
    }

    /// To string.
    pub fn to_string(&self) -> ArcStr {
        match self {
            Self::None => literal!(""),
            Self::Undefined => literal!("undefined"),

            // Angles.
            Self::PositiveRadians => literal!("prad"),
            Self::Radians => literal!("rad"),
            Self::PositiveDegrees => literal!("pdeg"),
            Self::Degrees => literal!("deg"),
            
            // Metric length.
            Self::Kilometers => literal!("km"),
            Self::Hectometers => literal!("hm"),
            Self::Decameters => literal!("dcm"),
            Self::Meters => literal!("m"),
            Self::Decimeters => literal!("dm"),
            Self::Centimeters => literal!("cm"),
            Self::Millimeters => literal!("mm"),
            Self::Micrometers => literal!("um"),
            Self::Nanometers => literal!("nm"),
    
            // Imperial length.
            Self::Miles => literal!("mi"),
            Self::Yards => literal!("yd"),
            Self::Feet => literal!("ft"),
            Self::Inches => literal!("in"),

            // Time units.
            Self::Days => literal!("days"),
            Self::Hours => literal!("hr"),
            Self::Minutes => literal!("min"),
            Self::Seconds => literal!("s"),
            Self::Milliseconds => literal!("ms"),
            Self::Microseconds => literal!("us"),
            Self::Nanoseconds => literal!("ns"),

            // Temperature units.
            Self::Kelvin => literal!("K"),
            Self::Celsius => literal!("C"),
            Self::Fahrenheit => literal!("F"),

            // Metric mass units.
            Self::Gigatonnes => literal!("Gt"),
            Self::Megatonnes => literal!("Mt"),
            Self::Tonnes => literal!("t"),
            Self::Kilograms => literal!("kg"),
            Self::Grams => literal!("g"),
            Self::Milligrams => literal!("mg"),
            Self::Micrograms => literal!("ug"),
            Self::Nanograms => literal!("ng"),
            Self::Picograms => literal!("pg"),

            // Imperial mass units (US).
            Self::Tons => literal!("Ton"),
            Self::Pounds => literal!("lb"),
            Self::Ounce => literal!("oz"),

            // Computer memory.
            Self::Bits => literal!("bits"),
            Self::Bytes => literal!("bytes"),
            Self::Kibibytes => literal!("KiB"),
            Self::Kilobytes => literal!("KB"),
            Self::Mebibytes => literal!("MiB"),
            Self::Megabytes => literal!("MB"),
            Self::Gibibytes => literal!("GiB"),
            Self::Gigabytes => literal!("GB"),
            Self::Tebibytes => literal!("TiB"),
            Self::Terabytes => literal!("TB"),
            Self::Pebibytes => literal!("PiB"),
            Self::Petabytes => literal!("PB"),
            Self::Exbibyte => literal!("EiB"),
            Self::Exabytes => literal!("EB"),
            Self::Zebibytes => literal!("ZiB"),
            Self::Zettabytes => literal!("ZB"),
            Self::Yobibytes => literal!("YiB"),
            Self::Yottabytes => literal!("YB"),
        }
    }

    /// Has units?
    pub fn has_units(&self) -> bool {
        match self {
            Self::None => false,
            _ => true,
        }
    }

    /// Is undefined?
    pub fn is_undefined(&self) -> bool {
        match self {
            Self::Undefined => true,
            _ => false,
        }
    }

    /// Is angle?
    pub fn is_angle(&self) -> bool {
        match self {
            Self::PositiveDegrees |
            Self::PositiveRadians |
            Self::Radians |
            Self::Degrees => true,
            _ => false,
        }
    }

    /// Is positive angle?
    pub fn is_positive_angle(&self) -> bool {
        match self {
            Self::PositiveDegrees |
            Self::PositiveRadians => true,
            _ => false,
        }
    }

    /// Is radians?
    pub fn is_radians(&self) -> bool {
        match self {
            Self::PositiveRadians |
            Self::Radians => true,
            _ => false,
        }
    }

    /// Is degrees?
    pub fn is_degrees(&self) -> bool {
        match self {
            Self::PositiveDegrees |
            Self::Degrees => true,
            _ =>  false,
        }
    }

    /// To radians.
    pub fn to_radians(value: f64, units: Self) -> f64 {
        match units {
            Self::PositiveDegrees |
            Self::Degrees => value*PI/180.0,
            _ => value,
        }
    }

    /// From radians.
    pub fn from_radians(rad: f64, units: Self) -> f64 {
        match units {
            Self::PositiveDegrees => {
                let mut degrees = rad*180.0/PI;
                
                // Make sure degrees are always between [0 and +-360)
                if degrees >= 360.   { degrees %= 360.; }
                if degrees <= -360.  { degrees = -(degrees.abs() % 360.); }

                // For this type, make sure degrees are always positive!
                if degrees < 0. {
                    ((360. + degrees)*1e6).round()/1e6 // 360 + -90 = 270, 360 + -270 = 90
                } else {
                    (degrees*1e6).round()/1e6
                }
            },
            Self::Degrees => {
                let mut degrees = rad*180.0/PI;
                
                // Make sure degrees are always between [0 and +-360)
                if degrees > 360.   { degrees %= 360.; }
                if degrees < -360.  { degrees = -(degrees.abs() % 360.); }

                (degrees*1e6).round()/1e6
            },
            Self::PositiveRadians => {
                let mut radians = rad;

                // Make sure radians are always between [0 and +-2PI)
                if radians >= PI*2.   { radians %= PI*2.; }
                if radians <= -PI*2.  { radians = -(radians.abs() % PI*2.); }

                // For this type, make sure radians are always positive!
                if radians < 0. {
                    (((PI*2.) + radians)*1e6).round()/1e6
                } else {
                    (radians*1e6).round()/1e6
                }
            },
            Self::Radians => {
                let mut radians = rad;

                // Make sure radians are always between [0 and +-2PI)
                if radians > PI*2.   { radians %= PI*2.; }
                if radians < -PI*2.  { radians = -(radians.abs() % PI*2.); }

                (radians*1e6).round()/1e6
            },
            _ => rad,
        }
    }

    /// Are two values in different units equal? Conversions round (Ex. 1.1km is 1100.0000000000002m), so values
    /// in different units compare equal within a relative 1e-14 (~45 ulps): far below any meaningful difference
    /// (1TB and 1TB + 1 byte differ by 1e-12). Values in the same units compare exactly.
    pub fn approx_eq(a: f64, b: f64) -> bool {
        a == b || (a - b).abs() <= 1e-14 * a.abs().max(b.abs())
    }

    /// Exact factor of a ratio unit relative to its dimension's base unit: (dimension, numerator, denominator).
    /// Bases: meters (length), seconds (time), grams (mass), bits (memory).
    fn factor(&self) -> Option<(u8, u128, u128)> {
        const LENGTH: u8 = 1;
        const TIME: u8 = 2;
        const MASS: u8 = 3;
        const MEMORY: u8 = 4;
        const K: u128 = 1024;
        Some(match self {
            Self::Kilometers => (LENGTH, 1000, 1),
            Self::Hectometers => (LENGTH, 100, 1),
            Self::Decameters => (LENGTH, 10, 1),
            Self::Meters => (LENGTH, 1, 1),
            Self::Decimeters => (LENGTH, 1, 10),
            Self::Centimeters => (LENGTH, 1, 100),
            Self::Millimeters => (LENGTH, 1, 1_000),
            Self::Micrometers => (LENGTH, 1, 1_000_000),
            Self::Nanometers => (LENGTH, 1, 1_000_000_000),
            Self::Miles => (LENGTH, 1_609_344, 1_000),   // 1609.344m
            Self::Yards => (LENGTH, 9_144, 10_000),      // 0.9144m
            Self::Feet => (LENGTH, 3_048, 10_000),       // 0.3048m
            Self::Inches => (LENGTH, 254, 10_000),       // 0.0254m

            Self::Days => (TIME, 86_400, 1),
            Self::Hours => (TIME, 3_600, 1),
            Self::Minutes => (TIME, 60, 1),
            Self::Seconds => (TIME, 1, 1),
            Self::Milliseconds => (TIME, 1, 1_000),
            Self::Microseconds => (TIME, 1, 1_000_000),
            Self::Nanoseconds => (TIME, 1, 1_000_000_000),

            Self::Gigatonnes => (MASS, 1_000_000_000_000_000, 1),
            Self::Megatonnes => (MASS, 1_000_000_000_000, 1),
            Self::Tonnes => (MASS, 1_000_000, 1),
            Self::Kilograms => (MASS, 1_000, 1),
            Self::Grams => (MASS, 1, 1),
            Self::Milligrams => (MASS, 1, 1_000),
            Self::Micrograms => (MASS, 1, 1_000_000),
            Self::Nanograms => (MASS, 1, 1_000_000_000),
            Self::Picograms => (MASS, 1, 1_000_000_000_000),
            Self::Tons => (MASS, 90_718_474, 100),          // US short ton: 907184.74g
            Self::Pounds => (MASS, 45_359_237, 100_000),     // 453.59237g
            Self::Ounce => (MASS, 45_359_237, 1_600_000),    // 1/16 lb

            Self::Bits => (MEMORY, 1, 1),
            Self::Bytes => (MEMORY, 8, 1),
            Self::Kilobytes => (MEMORY, 8 * 1_000, 1),
            Self::Kibibytes => (MEMORY, 8 * K, 1),
            Self::Megabytes => (MEMORY, 8 * 1_000_000, 1),
            Self::Mebibytes => (MEMORY, 8 * K * K, 1),
            Self::Gigabytes => (MEMORY, 8 * 1_000_000_000, 1),
            Self::Gibibytes => (MEMORY, 8 * K * K * K, 1),
            Self::Terabytes => (MEMORY, 8 * 1_000_000_000_000, 1),
            Self::Tebibytes => (MEMORY, 8 * K * K * K * K, 1),
            Self::Petabytes => (MEMORY, 8 * 1_000_000_000_000_000, 1),
            Self::Pebibytes => (MEMORY, 8 * K * K * K * K * K, 1),
            Self::Exabytes => (MEMORY, 8 * 1_000_000_000_000_000_000, 1),
            Self::Exbibyte => (MEMORY, 8 * K * K * K * K * K * K, 1),
            Self::Zettabytes => (MEMORY, 8 * 1_000_000_000_000_000_000_000, 1),
            Self::Zebibytes => (MEMORY, 8 * K * K * K * K * K * K * K, 1),
            Self::Yottabytes => (MEMORY, 8 * 1_000_000_000_000_000_000_000_000, 1),
            Self::Yobibytes => (MEMORY, 8 * K * K * K * K * K * K * K * K, 1),
            _ => return None,
        })
    }

    /// Scale a value by the exact ratio num/den in one rounding when possible
    /// (whole-number ratios multiply, unit-fraction ratios divide).
    fn scale(value: f64, num: u128, den: u128) -> f64 {
        fn gcd(mut a: u128, mut b: u128) -> u128 {
            while b != 0 { let t = a % b; a = b; b = t; }
            a
        }
        let g = gcd(num, den).max(1);
        let (num, den) = (num / g, den / g);
        if den == 1 {
            value * num as f64
        } else if num == 1 {
            value / den as f64
        } else {
            value * num as f64 / den as f64
        }
    }

    /// Is mass?
    pub fn is_mass(&self) -> bool {
        match self {
            Self::Gigatonnes |
            Self::Megatonnes |
            Self::Tonnes |
            Self::Kilograms |
            Self::Grams |
            Self::Milligrams |
            Self::Micrograms |
            Self::Nanograms |
            Self::Picograms |
            Self::Tons |
            Self::Pounds |
            Self::Ounce => true,
            _ => false,
        }
    }

    /// Is metric mass?
    pub fn is_metric_mass(&self) -> bool {
        match self {
            Self::Gigatonnes |
            Self::Megatonnes |
            Self::Tonnes |
            Self::Kilograms |
            Self::Grams |
            Self::Milligrams |
            Self::Micrograms |
            Self::Nanograms |
            Self::Picograms => true,
            _ => false,
        }
    }

    /// Is imperial mass?
    pub fn is_imperial_mass(&self) -> bool {
        match self {
            Self::Tons |
            Self::Pounds |
            Self::Ounce => true,
            _ => false,
        }
    }



    /// Is temperature?
    pub fn is_temperature(&self) -> bool {
        match self {
            Self::Kelvin |
            Self::Celsius |
            Self::Fahrenheit => true,
            _ => false,
        }
    }

    /// To C.
    fn to_c(value: f64, units: Self) -> f64 {
        match units {
            Self::Kelvin => value - 273.15,
            Self::Celsius => value,
            Self::Fahrenheit => (value - 32.0)*(5.0/9.0),
            _ => value,
        }
    }

    /// From C.
    fn from_c(value: f64, units: Self) -> f64 {
        match units {
            Self::Kelvin => value + 273.15,
            Self::Celsius => value,
            Self::Fahrenheit => (value * (9.0/5.0)) + 32.0,
            _ => value,
        }
    }

    /// Is time?
    pub fn is_time(&self) -> bool {
        match self {
            Self::Days |
            Self::Hours |
            Self::Minutes |
            Self::Seconds |
            Self::Milliseconds |
            Self::Microseconds |
            Self::Nanoseconds => true,
            _ => false,
        }
    }



    /// Is length?
    pub fn is_length(&self) -> bool {
        match self {
            Self::Kilometers |
            Self::Hectometers |
            Self::Decameters |
            Self::Meters |
            Self::Decimeters |
            Self::Centimeters | 
            Self::Millimeters |
            Self::Micrometers |
            Self::Nanometers |
            Self::Miles |
            Self::Yards |
            Self::Feet |
            Self::Inches => true,
            _ => false,
        }
    }

    /// Is metric length?
    pub fn is_metric_length(&self) -> bool {
        match self {
            Self::Kilometers |
            Self::Hectometers |
            Self::Decameters |
            Self::Meters |
            Self::Decimeters |
            Self::Centimeters | 
            Self::Millimeters |
            Self::Micrometers |
            Self::Nanometers => true,
            _ => false,
        }
    }

    /// Is imperial length?
    pub fn is_imperial_length(&self) -> bool {
        match self {
            Self::Miles |
            Self::Yards |
            Self::Feet |
            Self::Inches => true,
            _ => false,
        }
    }



    /// Is computer memory?
    pub fn is_memory(&self) -> bool {
        match self {
            Self::Bits |
            Self::Bytes |
            Self::Kibibytes |
            Self::Kilobytes |
            Self::Mebibytes |
            Self::Megabytes |
            Self::Gibibytes |
            Self::Gigabytes |
            Self::Tebibytes |
            Self::Terabytes |
            Self::Pebibytes |
            Self::Petabytes |
            Self::Exbibyte |
            Self::Exabytes |
            Self::Zebibytes |
            Self::Zettabytes |
            Self::Yobibytes |
            Self::Yottabytes => true,
            _ => false,
        }
    }


}
