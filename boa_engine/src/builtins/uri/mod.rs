//! URI Handling Functions
//!
//! Uniform Resource Identifiers, or URIs, are Strings that identify resources (e.g. web pages or
//! files) and transport protocols by which to access them (e.g. HTTP or FTP) on the Internet. The
//! ECMAScript language itself does not provide any support for using URIs except for functions
//! that encode and decode URIs as described in 19.2.6.2, 19.2.6.3, 19.2.6.4 and 19.2.6.5
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-uri-handling-functions

mod consts;

use self::consts::{
    is_uri_reserved_or_number_sign, is_uri_reserved_or_uri_unescaped_or_number_sign,
    is_uri_unescaped,
};

use super::{string::code_point_at, BuiltIn};
use crate::{
    builtins::JsArgs, object::FunctionBuilder, property::Attribute, Context, JsResult, JsString,
    JsValue,
};

/// URI Handling Functions
#[derive(Debug, Clone, Copy)]
pub(crate) struct Uri;

impl BuiltIn for Uri {
    const NAME: &'static str = "Uri";

    fn init(context: &mut Context) -> Option<JsValue> {
        let decode_uri = FunctionBuilder::native(context, Self::decode_uri)
            .name("decodeURI")
            .length(1)
            .constructor(false)
            .build();

        context.register_global_property(
            "decodeURI",
            decode_uri,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );

        let decode_uri_component = FunctionBuilder::native(context, Self::decode_uri_component)
            .name("decodeURIComponent")
            .length(1)
            .constructor(false)
            .build();

        context.register_global_property(
            "decodeURIComponent",
            decode_uri_component,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );

        let encode_uri = FunctionBuilder::native(context, Self::encode_uri)
            .name("encodeURI")
            .length(1)
            .constructor(false)
            .build();

        context.register_global_property(
            "encodeURI",
            encode_uri,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );

        let encode_uri_component = FunctionBuilder::native(context, Self::encode_uri_component)
            .name("encodeURIComponent")
            .length(1)
            .constructor(false)
            .build();

        context.register_global_property(
            "encodeURIComponent",
            encode_uri_component,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );

        None
    }
}

impl Uri {
    /// Builtin JavaScript `decodeURI ( encodedURI )` function.
    ///
    /// This function computes a new version of a URI in which each escape sequence and UTF-8
    /// encoding of the sort that might be introduced by the `encodeURI` function is replaced with
    /// the UTF-16 encoding of the code points that it represents. Escape sequences that could not
    /// have been introduced by `encodeURI` are not replaced.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-decodeuri-encodeduri
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/decodeURI
    pub(crate) fn decode_uri(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let encoded_uri = args.get_or_undefined(0);

        // 1. Let uriString be ? ToString(encodedURI).
        let uri_string = encoded_uri.to_string(context)?;

        // 2. Let reservedURISet be a String containing one instance of each code unit valid in uriReserved plus "#".
        let reserved_uri_set = is_uri_reserved_or_number_sign;

        // 3. Return ? Decode(uriString, reservedURISet).
        Ok(JsValue::from(decode(
            context,
            &uri_string,
            reserved_uri_set,
        )?))
    }

    /// Builtin JavaScript `decodeURIComponent ( encodedURIComponent )` function.
    ///
    /// This function computes a new version of a URI in which each escape sequence and UTF-8
    /// encoding of the sort that might be introduced by the `encodeURIComponent` function is
    /// replaced with the UTF-16 encoding of the code points that it represents.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-decodeuricomponent-encodeduricomponent
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/decodeURIComponent
    pub(crate) fn decode_uri_component(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let encoded_uri_component = args.get_or_undefined(0);

        // 1. Let componentString be ? ToString(encodedURIComponent).
        let component_string = encoded_uri_component.to_string(context)?;

        // 2. Let reservedURIComponentSet be the empty String.
        let reserved_uri_component_set = |_: u16| false;

        // 3. Return ? Decode(componentString, reservedURIComponentSet).
        Ok(JsValue::from(decode(
            context,
            &component_string,
            reserved_uri_component_set,
        )?))
    }

    /// Builtin JavaScript `encodeURI ( uri )` function.
    ///
    /// This function computes a new version of a UTF-16 encoded (6.1.4) URI in which each instance
    /// of certain code points is replaced by one, two, three, or four escape sequences
    /// representing the UTF-8 encoding of the code points.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-encodeuri-uri
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/encodeURI
    pub(crate) fn encode_uri(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let uri = args.get_or_undefined(0);

        // 1. Let uriString be ? ToString(uri).
        let uri_string = uri.to_string(context)?;

        // 2. Let unescapedURISet be a String containing one instance of each code unit valid in uriReserved and uriUnescaped plus "#".
        let unescaped_uri_set = is_uri_reserved_or_uri_unescaped_or_number_sign;

        // 3. Return ? Encode(uriString, unescapedURISet).
        Ok(JsValue::from(encode(
            context,
            &uri_string,
            unescaped_uri_set,
        )?))
    }

    /// Builtin JavaScript `encodeURIComponent ( uriComponent )` function.
    ///
    /// This function computes a new version of a UTF-16 encoded (6.1.4) URI in which each instance
    /// of certain code points is replaced by one, two, three, or four escape sequences
    /// representing the UTF-8 encoding of the code point.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-encodeuricomponent-uricomponent
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/encodeURIComponent
    pub(crate) fn encode_uri_component(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let uri_component = args.get_or_undefined(0);

        // 1. Let componentString be ? ToString(uriComponent).
        let component_string = uri_component.to_string(context)?;

        // 2. Let unescapedURIComponentSet be a String containing one instance of each code unit valid in uriUnescaped.
        let unescaped_uri_component_set = is_uri_unescaped;

        // 3. Return ? Encode(componentString, unescapedURIComponentSet).
        Ok(JsValue::from(encode(
            context,
            &component_string,
            unescaped_uri_component_set,
        )?))
    }
}

/// The `Encode ( string, unescapedSet )` abstract operation
///
/// The abstract operation Encode takes arguments `string` (a String) and `unescapedSet` (a String)
/// and returns either a normal completion containing a String or a throw completion. It performs
/// URI encoding and escaping.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-encode
fn encode<F>(context: &mut Context, string: &JsString, unescaped_set: F) -> JsResult<String>
where
    F: Fn(u16) -> bool,
{
    let code_units = string.encode_utf16().collect::<Vec<_>>();

    // 1. Let strLen be the length of string.
    let str_len = code_units.len();

    // 2. Let R be the empty String.
    let mut r = String::new();

    // 3. Let k be 0.
    let mut k = 0;
    // 4. Repeat,
    loop {
        // a. If k = strLen, return R.
        if k == str_len {
            return Ok(r);
        }

        // b. Let C be the code unit at index k within string.
        let c = code_units[k];

        // c. If C is in unescapedSet, then
        if unescaped_set(c) {
            // i. Set k to k + 1.
            k += 1;

            // ii. Set R to the string-concatenation of R and C.
            r.push(char::from_u32(u32::from(c)).expect("char from code point cannot fail here"));
        } else {
            // d. Else,
            // i. Let cp be CodePointAt(string, k).
            let cp = code_point_at(string, k as u64);

            // ii. If cp.[[IsUnpairedSurrogate]] is true, throw a URIError exception.
            if cp.is_unpaired_surrogate {
                context.throw_uri_error("trying to encode an invalid string")?;
            }

            // iii. Set k to k + cp.[[CodeUnitCount]].
            k += cp.code_unit_count as usize;

            // iv. Let Octets be the List of octets resulting by applying the UTF-8 transformation
            //     to cp.[[CodePoint]].
            let mut buff = [0_u8; 4]; // Will never be more than 4 bytes

            let octets = char::from_u32(cp.code_point)
                .expect("valid unicode code point to char conversion failed")
                .encode_utf8(&mut buff);

            // v. For each element octet of Octets, do
            for octet in octets.bytes() {
                // 1. Set R to the string-concatenation of:
                //    R
                //    "%"
                //    the String representation of octet, formatted as a two-digit uppercase
                //    hexadecimal number, padded to the left with a zero if necessary
                r = format!("{r}%{octet:0>2X}");
            }
        }
    }
}

/// The `Decode ( string, reservedSet )` abstract operation.
///
/// The abstract operation Decode takes arguments `string` (a String) and `reservedSet` (a String)
/// and returns either a normal completion containing a String or a throw completion. It performs
/// URI unescaping and decoding.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-decode
#[allow(clippy::many_single_char_names)]
fn decode<F>(context: &mut Context, string: &JsString, reserved_set: F) -> JsResult<String>
where
    F: Fn(u16) -> bool,
{
    let code_units = string.encode_utf16().collect::<Vec<_>>();

    // 1. Let strLen be the length of string.
    let str_len = code_units.len();
    // 2. Let R be the empty String.
    let mut r = Vec::new();

    // 3. Let k be 0.
    let mut k = 0;
    // 4. Repeat,
    loop {
        // a. If k = strLen, return R.
        if k == str_len {
            return Ok(String::from_utf16(&r).expect("invalid UTF-16 characters found"));
        }

        // b. Let C be the code unit at index k within string.
        let c = code_units[k];

        // c. If C is not the code unit 0x0025 (PERCENT SIGN), then
        #[allow(clippy::if_not_else)]
        let s = if c != 0x0025_u16 {
            // i. Let S be the String value containing only the code unit C.
            Vec::from([c])
        } else {
            // d. Else,
            // i. Let start be k.
            let start = k;

            // ii. If k + 2 ≥ strLen, throw a URIError exception.
            if k + 2 >= str_len {
                context.throw_uri_error("invalid escape character found")?;
            }

            // iii. If the code units at index (k + 1) and (k + 2) within string do not represent
            // hexadecimal digits, throw a URIError exception.
            // iv. Let B be the 8-bit value represented by the two hexadecimal digits at index (k + 1) and (k + 2).
            let b = decode_hex_byte(code_units[k + 1], code_units[k + 2])
                .ok_or_else(|| context.construct_uri_error("invalid hexadecimal digit found"))?;

            // v. Set k to k + 2.
            k += 2;

            // vi. Let n be the number of leading 1 bits in B.
            let n = leading_one_bits(b);

            // vii. If n = 0, then
            if n == 0 {
                // 1. Let C be the code unit whose value is B.
                let c = u16::from(b);

                // 2. If C is not in reservedSet, then
                if !reserved_set(c) {
                    // a. Let S be the String value containing only the code unit C.
                    Vec::from([c])
                } else {
                    // 3. Else,
                    // a. Let S be the substring of string from start to k + 1.
                    Vec::from(&code_units[start..=k])
                }
            } else {
                // viii. Else,
                // 1. If n = 1 or n > 4, throw a URIError exception.
                if n == 1 || n > 4 {
                    context.throw_uri_error("invalid escaped character found")?;
                }

                // 2. If k + (3 × (n - 1)) ≥ strLen, throw a URIError exception.
                if k + (3 * (n - 1)) > str_len {
                    context.throw_uri_error("non-terminated escape character found")?;
                }

                // 3. Let Octets be « B ».
                let mut octets = Vec::from([b]);

                // 4. Let j be 1.
                // 5. Repeat, while j < n,
                for _j in 1..n {
                    // a. Set k to k + 1.
                    k += 1;

                    // b. If the code unit at index k within string is not the code unit 0x0025 (PERCENT SIGN), throw a URIError exception.
                    if code_units[k] != 0x0025 {
                        context
                            .throw_uri_error("escape characters must be preceded with a % sign")?;
                    }

                    // c. If the code units at index (k + 1) and (k + 2) within string do not represent hexadecimal digits, throw a URIError exception.
                    // d. Let B be the 8-bit value represented by the two hexadecimal digits at index (k + 1) and (k + 2).
                    let b =
                        decode_hex_byte(code_units[k + 1], code_units[k + 2]).ok_or_else(|| {
                            context.construct_uri_error("invalid hexadecimal digit found")
                        })?;

                    // e. Set k to k + 2.
                    k += 2;

                    // f. Append B to Octets.
                    octets.push(b);

                    // g. Set j to j + 1.
                }

                // 6. Assert: The length of Octets is n.
                assert_eq!(octets.len(), n);

                // 7. If Octets does not contain a valid UTF-8 encoding of a Unicode code point, throw a URIError exception.
                match String::from_utf8(octets) {
                    Err(_) => {
                        return Err(context.construct_uri_error("invalid UTF-8 encoding found"))
                    }
                    Ok(v) => {
                        // 8. Let V be the code point obtained by applying the UTF-8 transformation to Octets, that is, from a List of octets into a 21-bit value.

                        // 9. Let S be UTF16EncodeCodePoint(V).
                        // utf16_encode_codepoint(v)
                        v.encode_utf16().collect::<Vec<_>>()
                    }
                }
            }
        };

        // e. Set R to the string-concatenation of R and S.
        r.extend_from_slice(&s);

        // f. Set k to k + 1.
        k += 1;
    }
}

/// Decodes a byte from two unicode code units.
fn decode_hex_byte(high: u16, low: u16) -> Option<u8> {
    match (
        char::from_u32(u32::from(high)),
        char::from_u32(u32::from(low)),
    ) {
        (Some(high), Some(low)) => match (high.to_digit(16), low.to_digit(16)) {
            (Some(high), Some(low)) => Some(((high as u8) << 4) + low as u8),
            _ => None,
        },
        _ => None,
    }
}

/// Counts the number of leading 1 bits in a given byte.
#[inline]
fn leading_one_bits(byte: u8) -> usize {
    // This uses a value table for speed
    if byte == u8::MAX {
        8
    } else if byte == 0b1111_1110 {
        7
    } else if byte & 0b1111_1100 == 0b1111_1100 {
        6
    } else if byte & 0b1111_1000 == 0b1111_1000 {
        5
    } else if byte & 0b1111_0000 == 0b1111_0000 {
        4
    } else if byte & 0b1110_0000 == 0b1110_0000 {
        3
    } else if byte & 0b1100_0000 == 0b1100_0000 {
        2
    } else if byte & 0b1000_0000 == 0b1000_0000 {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Checks if the `leading_one_bits()` function works as expected.
    #[test]
    fn ut_leading_one_bits() {
        assert_eq!(leading_one_bits(0b1111_1111), 8);
        assert_eq!(leading_one_bits(0b1111_1110), 7);

        assert_eq!(leading_one_bits(0b1111_1100), 6);
        assert_eq!(leading_one_bits(0b1111_1101), 6);

        assert_eq!(leading_one_bits(0b1111_1011), 5);
        assert_eq!(leading_one_bits(0b1111_1000), 5);

        assert_eq!(leading_one_bits(0b1111_0000), 4);
        assert_eq!(leading_one_bits(0b1111_0111), 4);

        assert_eq!(leading_one_bits(0b1110_0000), 3);
        assert_eq!(leading_one_bits(0b1110_1111), 3);

        assert_eq!(leading_one_bits(0b1100_0000), 2);
        assert_eq!(leading_one_bits(0b1101_1111), 2);

        assert_eq!(leading_one_bits(0b1000_0000), 1);
        assert_eq!(leading_one_bits(0b1011_1111), 1);

        assert_eq!(leading_one_bits(0b0000_0000), 0);
        assert_eq!(leading_one_bits(0b0111_1111), 0);
    }

    /// Checks that the `decode_byte()` function works as expected.
    #[test]
    fn ut_decode_byte() {
        // Sunny day tests
        assert_eq!(
            decode_hex_byte(u16::from(b'2'), u16::from(b'0')).unwrap(),
            0x20
        );
        assert_eq!(
            decode_hex_byte(u16::from(b'2'), u16::from(b'A')).unwrap(),
            0x2A
        );
        assert_eq!(
            decode_hex_byte(u16::from(b'3'), u16::from(b'C')).unwrap(),
            0x3C
        );
        assert_eq!(
            decode_hex_byte(u16::from(b'4'), u16::from(b'0')).unwrap(),
            0x40
        );
        assert_eq!(
            decode_hex_byte(u16::from(b'7'), u16::from(b'E')).unwrap(),
            0x7E
        );
        assert_eq!(
            decode_hex_byte(u16::from(b'0'), u16::from(b'0')).unwrap(),
            0x00
        );

        // Rainy day tests
        assert!(decode_hex_byte(u16::from(b'-'), u16::from(b'0')).is_none());
        assert!(decode_hex_byte(u16::from(b'f'), u16::from(b'~')).is_none());
        assert!(decode_hex_byte(u16::from(b'A'), 0_u16).is_none());
        assert!(decode_hex_byte(u16::from(b'%'), u16::from(b'&')).is_none());

        assert!(decode_hex_byte(0xFACD_u16, u16::from(b'-')).is_none());
        assert!(decode_hex_byte(u16::from(b'-'), 0xA0FD_u16).is_none());
    }
}
