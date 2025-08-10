//
// Copyright 2024 Formata, Inc. All rights reserved.
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

use nom::{branch::alt, bytes::complete::{tag, take_while1}, character::complete::one_of, combinator::{opt, recognize, value}, multi::{many0, many1}, IResult, Parser};
use crate::{parser::{doc::StofParseError, whitespace::whitespace}, runtime::{Num, Units, Val}};


/// Parse a number value (consumes whitespace up front).
pub fn number(input: &str) -> IResult<&str, Val, StofParseError> {
    let (input , _) = whitespace(input)?;
    let input = input.trim_start_matches("+");

    // binary
    if input.starts_with("0b") || input.starts_with("-0b") {
        let (input, binary) = take_while1(|c| c == '1' || c == '0' || c == 'b' || c == '_' || c == '-').parse(input)?;
        let bin_input = binary.replace("0b", "").replace('_', "");
        let val = i64::from_str_radix(&bin_input, 2).expect("failed to parse binary number");

        let (input, units) = units(input)?;
        if let Some(units) = units {
            return Ok((input, Val::Num(Num::Units(val as f64, units))));
        }
        return Ok((input, Val::Num(Num::Int(val))));
    }
    // oct
    if input.starts_with("0o") || input.starts_with("-0o") {
        let (input, oct) = take_while1(|c|
            c == '0' || c == '1' || c == '2' || c == '3' || c == '4' || c == '5' || c == '6' || c == '7' || c == 'o' || c == '_' || c == '-').parse(input)?;
        let oct_input = oct.replace("0o", "").replace('_', "");
        let val = i64::from_str_radix(&oct_input, 8).expect("failed to parse oct number");

        let (input, units) = units(input)?;
        if let Some(units) = units {
            return Ok((input, Val::Num(Num::Units(val as f64, units))));
        }
        return Ok((input, Val::Num(Num::Int(val))));
    }
    // hex
    if input.starts_with("0x") || input.starts_with("-0x") {
        let (input, hex) = take_while1(|c|
            c == '0' || c == '1' || c == '2' || c == '3' || c == '4' || c == '5' || c == '6' || c == '7' || c == '8' || c == '9' ||
            c == 'a' || c == 'A' || c == 'b' || c == 'B' || c == 'c' || c == 'C' || c == 'd' || c == 'D' || c == 'e' || c == 'E' || c == 'f' || c == 'F' || c == 'x' || c == '_' || c == '-').parse(input)?;
        let hex_input = hex.replace("0x", "").replace('_', "");
        let val = i64::from_str_radix(&hex_input, 16).expect("failed to parse hex number");
        
        let (input, units) = units(input)?;
        if let Some(units) = units {
            return Ok((input, Val::Num(Num::Units(val as f64, units))));
        }
        return Ok((input, Val::Num(Num::Int(val))));
    }

    let (input, recognized_float_str) = recognize(
(
            opt(tag("-")),
            many1(one_of("0123456789_")),
            opt(
                (
                    tag("."),
                    many0(one_of("0123456789_"))
                )
            ),
            opt((
                alt((tag("e"), tag("E"))),
                opt(alt((tag("+"), tag("-")))),
                many1(one_of("0123456789_")),
            ))
        )
    ).parse(input)?;

    let cleaned_string = recognized_float_str.replace("_", "");
    let float_value = cleaned_string.parse::<f64>().expect("could not parse floating point number");

    let (input, units) = units(input)?;
    if let Some(units) = units {
        return Ok((input, Val::Num(Num::Units(float_value, units))));
    }
    if !recognized_float_str.contains('.') && float_value.fract().abs() < 1e-10 {
        Ok((input, Val::Num(Num::Int(float_value.trunc() as i64))))
    } else {
        Ok((input, Val::Num(Num::Float(float_value))))
    }
}

// Parse optional units.
fn units(input: &str) -> IResult<&str, Option<Units>, StofParseError> {
    opt(
        alt([
            value(Units::Radians, alt((tag("radians"), tag("rad")))),
            value(Units::Degrees, alt((tag("degrees"), tag("deg")))),
            value(Units::PositiveRadians, alt((tag("pradians"), tag("prad")))),
            value(Units::PositiveDegrees, alt((tag("pdegrees"), tag("pdeg")))),

            value(Units::Kilometers, alt((tag("km"), tag("kilometers")))),
            value(Units::Hectometers, alt((tag("hm"), tag("hectometers")))),
            value(Units::Decameters, alt((tag("dcm"), tag("decameters")))),
            value(Units::Decimeters, alt((tag("dm"), tag("decimeters")))),
            value(Units::Centimeters, alt((tag("cm"), tag("centimeters")))),
            value(Units::Millimeters, alt((tag("mm"), tag("millimeters")))),
            value(Units::Micrometers, alt((tag("um"), tag("micrometers")))),
            value(Units::Nanometers, alt((tag("nm"), tag("nanometers")))),

            value(Units::Yards, alt((tag("yd"), tag("yards")))),
            value(Units::Feet, alt((tag("ft"), tag("feet")))),
            value(Units::Inches, alt((tag("inches"), tag("in")))),

            value(Units::Days, alt((tag("days"), tag("day")))),
            value(Units::Hours, alt((tag("hr"), tag("hours")))),
            value(Units::Minutes, alt((tag("minutes"), tag("min")))),
            value(Units::Milliseconds, alt((tag("ms"), tag("milliseconds")))),
            value(Units::Microseconds, alt((tag("us"), tag("microseconds")))),
            value(Units::Nanoseconds, alt((tag("ns"), tag("nanoseconds")))),

            value(Units::Kelvin, alt((tag("K"), tag("kelvin")))),
            value(Units::Celsius, alt((tag("C"), tag("celsius")))),
            value(Units::Fahrenheit, alt((tag("F"), tag("fahrenheit")))),

            value(Units::Gigatonnes, alt((tag("Gt"), tag("gigatonnes")))),
            value(Units::Megatonnes, alt((tag("Mt"), tag("megatonnes")))),
            value(Units::Kilograms, alt((tag("kg"), tag("kilograms")))),
            value(Units::Milligrams, alt((tag("mg"), tag("milligrams")))),
            value(Units::Micrograms, alt((tag("ug"), tag("micrograms")))),
            value(Units::Nanograms, alt((tag("ng"), tag("nanograms")))),
            value(Units::Picograms, alt((tag("pg"), tag("picograms")))),

            value(Units::Tons, alt((tag("tons"), tag("Ton")))),
            value(Units::Pounds, alt((tag("lbs"), tag("lb")))),
            value(Units::Ounce, alt((tag("oz"), tag("ounces")))),

            value(Units::Grams, alt((tag("grams"), tag("g")))),
            value(Units::Tonnes, alt((tag("tonnes"), tag("t")))),
            value(Units::Seconds, alt((tag("seconds"), tag("s")))),
            value(Units::Miles, alt((tag("miles"), tag("mi")))),
            value(Units::Meters, alt((tag("meters"), tag("m")))),
        ])
    ).parse(input)
}


#[cfg(test)]
mod tests {
    use crate::{parser::number::number, runtime::{Units, Val}};

    #[test]
    fn integer() {
        let val = number("4_005").unwrap();
        assert_eq!(val.1, Val::from(4005));
    }

    #[test]
    fn float() {
        let val = number("4_005.").unwrap();
        assert_eq!(val.1, Val::from(4005.0));
    }

    #[test]
    fn integer_units() {
        let val = number("+4_005mg").unwrap();
        assert_eq!(val.1, Val::from((4.005, Units::Grams)));
    }

    #[test]
    fn binary() {
        let val = number("0b0011").unwrap();
        assert_eq!(val.1, Val::from(3));
    }

    #[test]
    fn octal() {
        let val = number("0o4_g").unwrap();
        assert_eq!(val.1, Val::from((4000, Units::Milligrams)));
    }

    #[test]
    fn hex() {
        let val = number("+0xA").unwrap();
        assert_eq!(val.1, Val::from(10));
    }

    #[test]
    fn nbinary() {
        let val = number("-0b0011").unwrap();
        assert_eq!(val.1, Val::from(-3));
    }

    #[test]
    fn noctal() {
        let val = number("-0o4").unwrap();
        assert_eq!(val.1, Val::from(-4));
    }

    #[test]
    fn nhex() {
        let val = number("-0xA").unwrap();
        assert_eq!(val.1, Val::from(-10));
    }

    #[test]
    fn units() {
        assert_eq!(number("1rad").unwrap().1.to_string(), "1rad");
        assert_eq!(number("1radians").unwrap().1.to_string(), "1rad");
        assert_eq!(number("1deg").unwrap().1.to_string(), "1deg");
        assert_eq!(number("1degrees").unwrap().1.to_string(), "1deg");
        assert_eq!(number("1prad").unwrap().1.to_string(), "1prad");
        assert_eq!(number("1pradians").unwrap().1.to_string(), "1prad");
        assert_eq!(number("1pdeg").unwrap().1.to_string(), "1pdeg");
        assert_eq!(number("1pdegrees").unwrap().1.to_string(), "1pdeg");

        assert_eq!(number("1km").unwrap().1.to_string(), "1km");
        assert_eq!(number("1kilometers").unwrap().1.to_string(), "1km");
        assert_eq!(number("1hm").unwrap().1.to_string(), "1hm");
        assert_eq!(number("1hectometers").unwrap().1.to_string(), "1hm");
        assert_eq!(number("1dcm").unwrap().1.to_string(), "1dcm");
        assert_eq!(number("1decameters").unwrap().1.to_string(), "1dcm");
        assert_eq!(number("1m").unwrap().1.to_string(), "1m");
        assert_eq!(number("1meters").unwrap().1.to_string(), "1m");
        assert_eq!(number("1dm").unwrap().1.to_string(), "1dm");
        assert_eq!(number("1decimeters").unwrap().1.to_string(), "1dm");
        assert_eq!(number("1cm").unwrap().1.to_string(), "1cm");
        assert_eq!(number("1centimeters").unwrap().1.to_string(), "1cm");
        assert_eq!(number("1mm").unwrap().1.to_string(), "1mm");
        assert_eq!(number("1millimeters").unwrap().1.to_string(), "1mm");
        assert_eq!(number("1um").unwrap().1.to_string(), "1um");
        assert_eq!(number("1micrometers").unwrap().1.to_string(), "1um");
        assert_eq!(number("1nm").unwrap().1.to_string(), "1nm");
        assert_eq!(number("1nanometers").unwrap().1.to_string(), "1nm");

        assert_eq!(number("1mi").unwrap().1.to_string(), "1mi");
        assert_eq!(number("1miles").unwrap().1.to_string(), "1mi");
        assert_eq!(number("1yd").unwrap().1.to_string(), "1yd");
        assert_eq!(number("1yards").unwrap().1.to_string(), "1yd");
        assert_eq!(number("1ft").unwrap().1.to_string(), "1ft");
        assert_eq!(number("1feet").unwrap().1.to_string(), "1ft");
        assert_eq!(number("1in").unwrap().1.to_string(), "1in");
        assert_eq!(number("1inches").unwrap().1.to_string(), "1in");

        assert_eq!(number("1days").unwrap().1.to_string(), "1days");
        assert_eq!(number("1day").unwrap().1.to_string(), "1days");
        assert_eq!(number("1hr").unwrap().1.to_string(), "1hr");
        assert_eq!(number("1hours").unwrap().1.to_string(), "1hr");
        assert_eq!(number("1min").unwrap().1.to_string(), "1min");
        assert_eq!(number("1minutes").unwrap().1.to_string(), "1min");
        assert_eq!(number("1s").unwrap().1.to_string(), "1s");
        assert_eq!(number("1seconds").unwrap().1.to_string(), "1s");
        assert_eq!(number("1ms").unwrap().1.to_string(), "1ms");
        assert_eq!(number("1milliseconds").unwrap().1.to_string(), "1ms");
        assert_eq!(number("1us").unwrap().1.to_string(), "1us");
        assert_eq!(number("1microseconds").unwrap().1.to_string(), "1us");
        assert_eq!(number("1ns").unwrap().1.to_string(), "1ns");
        assert_eq!(number("1nanoseconds").unwrap().1.to_string(), "1ns");

        assert_eq!(number("1K").unwrap().1.to_string(), "1K");
        assert_eq!(number("1kelvin").unwrap().1.to_string(), "1K");
        assert_eq!(number("1C").unwrap().1.to_string(), "1C");
        assert_eq!(number("1celsius").unwrap().1.to_string(), "1C");
        assert_eq!(number("1F").unwrap().1.to_string(), "1F");
        assert_eq!(number("1fahrenheit").unwrap().1.to_string(), "1F");

        assert_eq!(number("1Gt").unwrap().1.to_string(), "1Gt");
        assert_eq!(number("1gigatonnes").unwrap().1.to_string(), "1Gt");
        assert_eq!(number("1Mt").unwrap().1.to_string(), "1Mt");
        assert_eq!(number("1megatonnes").unwrap().1.to_string(), "1Mt");
        assert_eq!(number("1t").unwrap().1.to_string(), "1t");
        assert_eq!(number("1tonnes").unwrap().1.to_string(), "1t");
        assert_eq!(number("1kg").unwrap().1.to_string(), "1kg");
        assert_eq!(number("1kilograms").unwrap().1.to_string(), "1kg");
        assert_eq!(number("1g").unwrap().1.to_string(), "1g");
        assert_eq!(number("1grams").unwrap().1.to_string(), "1g");
        assert_eq!(number("1mg").unwrap().1.to_string(), "1mg");
        assert_eq!(number("1milligrams").unwrap().1.to_string(), "1mg");
        assert_eq!(number("1ug").unwrap().1.to_string(), "1ug");
        assert_eq!(number("1micrograms").unwrap().1.to_string(), "1ug");
        assert_eq!(number("1ng").unwrap().1.to_string(), "1ng");
        assert_eq!(number("1nanograms").unwrap().1.to_string(), "1ng");
        assert_eq!(number("1pg").unwrap().1.to_string(), "1pg");
        assert_eq!(number("1picograms").unwrap().1.to_string(), "1pg");

        assert_eq!(number("1Ton").unwrap().1.to_string(), "1Ton");
        assert_eq!(number("1tons").unwrap().1.to_string(), "1Ton");
        assert_eq!(number("1lb").unwrap().1.to_string(), "1lb");
        assert_eq!(number("1lbs").unwrap().1.to_string(), "1lb");
        assert_eq!(number("1oz").unwrap().1.to_string(), "1oz");
        assert_eq!(number("1ounces").unwrap().1.to_string(), "1oz");
    }
}
