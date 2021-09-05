//! Parse WebAssembly types encoded in the binary format.
//!
//! See <https://webassembly.github.io/spec/core/binary/types.html>

use crate::parser::values::parse_vector;
use crate::{
    FloatType, FunctionType, IntegerType, NumberType, ReferenceType, ResultType, ValueType,
};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::sequence::{preceded, tuple};
use nom::IResult;

/// Parses a WebAssembly integer type from the input.
///
/// See <https://webassembly.github.io/spec/core/binary/types.html#number-types>
pub fn parse_integer_type(input: &[u8]) -> IResult<&[u8], IntegerType> {
    alt((
        map(tag(&[0x7F]), |_| IntegerType::I32),
        map(tag(&[0x7E]), |_| IntegerType::I64),
    ))(input)
}

/// Parses a WebAssembly float type from the input.
///
/// See <https://webassembly.github.io/spec/core/binary/types.html#number-types>
pub fn parse_float_type(input: &[u8]) -> IResult<&[u8], FloatType> {
    alt((
        map(tag(&[0x7D]), |_| FloatType::F32),
        map(tag(&[0x7C]), |_| FloatType::F64),
    ))(input)
}

/// Parses a WebAssembly number type from the input.
///
/// See <https://webassembly.github.io/spec/core/binary/types.html#number-types>
pub fn parse_number_type(input: &[u8]) -> IResult<&[u8], NumberType> {
    alt((
        map(parse_integer_type, NumberType::from),
        map(parse_float_type, NumberType::from),
    ))(input)
}

/// Parses a WebAssembly reference type from the input.
///
/// See <https://webassembly.github.io/spec/core/binary/types.html#reference-types>
pub fn parse_reference_type(input: &[u8]) -> IResult<&[u8], ReferenceType> {
    alt((
        map(tag(&[0x70]), |_| ReferenceType::Function),
        map(tag(&[0x6F]), |_| ReferenceType::External),
    ))(input)
}

/// Parses a WebAssembly value type from the input.
///
/// See <https://webassembly.github.io/spec/core/binary/types.html#value-types>
pub fn parse_value_type(input: &[u8]) -> IResult<&[u8], ValueType> {
    alt((
        map(parse_number_type, ValueType::from),
        map(parse_reference_type, ValueType::from),
    ))(input)
}

/// Parses a WebAssembly result type from the input.
///
/// See <https://webassembly.github.io/spec/core/binary/types.html#result-types>
pub fn parse_result_type(input: &[u8]) -> IResult<&[u8], ResultType> {
    map(parse_vector(parse_value_type), ResultType::from)(input)
}

/// Parses a WebAssembly function type from the input.
///
/// See <https://webassembly.github.io/spec/core/binary/types.html#function-types>
pub fn parse_function_type(input: &[u8]) -> IResult<&[u8], FunctionType> {
    map(
        preceded(tag(&[0x60]), tuple((parse_result_type, parse_result_type))),
        |(parameters, results)| FunctionType::new(parameters, results),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_result_type_with_all_types() {
        let extra = 0x01;
        let mut input = vec![0x7F, 0x7E, 0x7D, 0x7C, 0x70, 0x6F];
        input.insert(0, input.len() as u8);
        input.push(extra);

        let (remaining, types) = parse_result_type(input.as_slice()).unwrap();
        let expected = vec![
            ValueType::I32,
            ValueType::I64,
            ValueType::F32,
            ValueType::F64,
            ValueType::FunctionReference,
            ValueType::ExternalReference,
        ]
        .into();

        assert_eq!(types, expected);
        assert_eq!(remaining, &[extra]);
    }

    #[test]
    fn parse_simple_function_type() {
        let extra = 0x01;
        let input = vec![0x60, 1, 0x7F, 1, 0x7D, extra];
        let (remaining, function_type) = parse_function_type(input.as_slice()).unwrap();
        let expected = FunctionType::new(vec![ValueType::I32].into(), vec![ValueType::F32].into());

        assert_eq!(function_type, expected);
        assert_eq!(remaining, &[extra]);
    }
}
