/// Determines if `input` is a valid hex digit.
#[inline]
#[must_use]
pub fn is_hex(input: u8) -> bool {
    input.is_ascii_digit() || (b'A'..=b'F').contains(&input) || (b'a'..=b'f').contains(&input)
}

/// Converts a hex digit to its numeric value.
#[inline]
#[must_use]
pub const fn hex_val(input: u8) -> u8 {
    (input & 0xF) + (input >> 6) * 9
}

#[cfg(test)]
#[allow(clippy::too_many_lines)]
mod util_tests {
    use super::*;

    #[test]
    fn test_is_valid_pattern_digit() {
        assert!(is_hex(b'0'));
        assert!(is_hex(b'1'));
        assert!(is_hex(b'2'));
        assert!(is_hex(b'3'));
        assert!(is_hex(b'4'));
        assert!(is_hex(b'5'));
        assert!(is_hex(b'6'));
        assert!(is_hex(b'7'));
        assert!(is_hex(b'8'));
        assert!(is_hex(b'9'));
        assert!(is_hex(b'A'));
        assert!(is_hex(b'B'));
        assert!(is_hex(b'C'));
        assert!(is_hex(b'D'));
        assert!(is_hex(b'E'));
        assert!(is_hex(b'F'));
        assert!(is_hex(b'a'));
        assert!(is_hex(b'b'));
        assert!(is_hex(b'c'));
        assert!(is_hex(b'd'));
        assert!(is_hex(b'e'));
        assert!(is_hex(b'f'));
        assert!(!is_hex(b'?'));
        assert!(!is_hex(b'G'));
        assert!(!is_hex(b'g'));
        assert!(!is_hex(b'.'));
    }

    #[test]
    fn test_hex_val() {
        assert_eq!(hex_val(b'0'), 0);
        assert_eq!(hex_val(b'1'), 1);
        assert_eq!(hex_val(b'2'), 2);
        assert_eq!(hex_val(b'3'), 3);
        assert_eq!(hex_val(b'4'), 4);
        assert_eq!(hex_val(b'5'), 5);
        assert_eq!(hex_val(b'6'), 6);
        assert_eq!(hex_val(b'7'), 7);
        assert_eq!(hex_val(b'8'), 8);
        assert_eq!(hex_val(b'9'), 9);
        assert_eq!(hex_val(b'A'), 10);
        assert_eq!(hex_val(b'B'), 11);
        assert_eq!(hex_val(b'C'), 12);
        assert_eq!(hex_val(b'D'), 13);
        assert_eq!(hex_val(b'E'), 14);
        assert_eq!(hex_val(b'F'), 15);
        assert_eq!(hex_val(b'a'), 10);
        assert_eq!(hex_val(b'b'), 11);
        assert_eq!(hex_val(b'c'), 12);
        assert_eq!(hex_val(b'd'), 13);
        assert_eq!(hex_val(b'e'), 14);
        assert_eq!(hex_val(b'f'), 15);
    }
}
