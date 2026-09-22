use crate::util::{hex_val, is_hex};

/// Represents an error for invalid bitmask.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvalidMaskError(u8);
impl std::error::Error for InvalidMaskError {}
impl std::fmt::Display for InvalidMaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid mask: {:02X}", self.0)
    }
}

/// Represents an error for invalid byte string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvalidByteStringError(String);
impl std::error::Error for InvalidByteStringError {}
impl std::fmt::Display for InvalidByteStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid byte string: {}", self.0)
    }
}

/// Represents the bitmask used for matching a byte's nibbles.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mask {
    /// Matches the entire byte (e.g. `0xAB`).
    #[default]
    Full = 0xFF,

    /// Matches the high nibble (e.g. `0xA?`).
    HighOnly = 0xF0,

    /// Matches the low nibble (e.g. `0x?B`).
    LowOnly = 0x0F,

    /// Ignore the entire byte (e.g. `0x??`).
    Wildcard = 0x00,
}

impl std::ops::BitAnd<Mask> for u8 {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Mask) -> Self::Output {
        self & (rhs as Self)
    }
}

impl std::ops::BitAnd<Mask> for &u8 {
    type Output = u8;

    #[inline]
    fn bitand(self, rhs: Mask) -> Self::Output {
        *self & (rhs as u8)
    }
}

impl std::ops::BitAnd for Mask {
    type Output = u8;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        (self as u8) & (rhs as u8)
    }
}

impl TryFrom<u8> for Mask {
    type Error = InvalidMaskError;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0xFF => Ok(Self::Full),
            0xF0 => Ok(Self::HighOnly),
            0x0F => Ok(Self::LowOnly),
            0x00 => Ok(Self::Wildcard),
            _ => Err(InvalidMaskError(value)),
        }
    }
}

/// Represents a byte with a mask.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PatternByte {
    pub byte: u8,
    pub mask: Mask,
}

impl std::fmt::Display for PatternByte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.mask {
            Mask::Full => write!(f, "{:02X}", self.byte),
            Mask::HighOnly => write!(f, "{:X}?", self.byte >> 4),
            Mask::LowOnly => write!(f, "?{:X}", self.byte & 0x0F),
            Mask::Wildcard => write!(f, "??"),
        }
    }
}

impl From<u8> for PatternByte {
    #[inline]
    fn from(value: u8) -> Self {
        Self {
            byte: value,
            mask: Mask::Full,
        }
    }
}

impl std::str::FromStr for PatternByte {
    type Err = InvalidByteStringError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        let s = trimmed
            .strip_prefix("0x")
            .or_else(|| trimmed.strip_prefix("0X"))
            .unwrap_or(trimmed);
        Self::try_from(s.as_bytes())
    }
}

impl TryFrom<&[u8]> for PatternByte {
    type Error = InvalidByteStringError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Ok(Self::default());
        }
        let (h, l) = match value {
            [b] => (b'0', *b),
            [h, l] => (*h, *l),
            _ => return Err(InvalidByteStringError(format!("Not a byte: {value:?}"))),
        };

        let (byte, mask) = match (h, l) {
            (b'?', b'?') => (0, Mask::Wildcard),
            (h, b'?') if is_hex(h) => (hex_val(h) << 4, Mask::HighOnly),
            (b'?', l) if is_hex(l) => (hex_val(l), Mask::LowOnly),
            (h, l) if is_hex(h) && is_hex(l) => ((hex_val(h) << 4) | hex_val(l), Mask::Full),
            _ => {
                return Err(InvalidByteStringError(format!("Invalid pattern: {value:?}")));
            },
        };
        Ok(Self { byte, mask })
    }
}

impl PartialEq<u8> for PatternByte {
    #[inline]
    fn eq(&self, other: &u8) -> bool {
        self.apply() == (other & self.mask)
    }
}

impl PartialOrd<u8> for PatternByte {
    #[inline]
    fn partial_cmp(&self, other: &u8) -> Option<std::cmp::Ordering> {
        self.apply().partial_cmp(&(other & self.mask))
    }
}

impl PatternByte {
    /// Creates a new `PatternByte`.
    #[inline]
    #[must_use]
    pub const fn new(byte: u8, mask: Mask) -> Self {
        Self { byte, mask }
    }

    /// Returns the stored byte with the mask applied.
    #[inline]
    #[must_use]
    pub fn apply(&self) -> u8 {
        self.byte & self.mask
    }

    /// Merges this `PatternByte` into a broader pattern.
    #[inline]
    #[must_use]
    pub fn merge(&self, other: Self) -> Self {
        let common = self.mask & other.mask & !(self.byte ^ other.byte);
        let high = (common & Mask::HighOnly) == 0xF0;
        let low = (common & Mask::LowOnly) == 0x0F;
        let mask = (u8::from(high) * 0xF0) | (u8::from(low) * 0x0F);
        Self {
            byte: self.byte,
            mask: Mask::try_from(mask).unwrap_or(Mask::Wildcard),
        }
    }
}

#[cfg(test)]
#[allow(clippy::too_many_lines)]
mod mask_tests {
    use super::*;

    #[test]
    fn test_bit_and_u8() {
        assert_eq!(0xAB & Mask::Full, 0xAB);
        assert_eq!(0xAB & Mask::HighOnly, 0xA0);
        assert_eq!(0xAB & Mask::LowOnly, 0x0B);
        assert_eq!(0xAB & Mask::Wildcard, 0x00);
    }

    #[test]
    fn test_bit_and_mask() {
        assert_eq!(Mask::Full & Mask::Full, 0xFF);
        assert_eq!(Mask::Full & Mask::HighOnly, 0xF0);
        assert_eq!(Mask::Full & Mask::LowOnly, 0x0F);
        assert_eq!(Mask::Full & Mask::Wildcard, 0x00);
        assert_eq!(Mask::HighOnly & Mask::HighOnly, 0xF0);
        assert_eq!(Mask::HighOnly & Mask::LowOnly, 0x00);
        assert_eq!(Mask::HighOnly & Mask::Wildcard, 0x00);
        assert_eq!(Mask::LowOnly & Mask::LowOnly, 0x0F);
        assert_eq!(Mask::LowOnly & Mask::Wildcard, 0x00);
        assert_eq!(Mask::Wildcard & Mask::Wildcard, 0x00);
    }

    #[test]
    fn test_try_from_u8() {
        assert_eq!(Mask::try_from(0xFF), Ok(Mask::Full));
        assert_eq!(Mask::try_from(0xF0), Ok(Mask::HighOnly));
        assert_eq!(Mask::try_from(0x0F), Ok(Mask::LowOnly));
        assert_eq!(Mask::try_from(0x00), Ok(Mask::Wildcard));
        assert_eq!(Mask::try_from(0x12), Err(InvalidMaskError(0x12)));
    }
}

#[cfg(test)]
#[allow(clippy::too_many_lines)]
mod pattern_byte_tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_display() {
        assert_eq!(
            PatternByte {
                byte: 0xAB,
                mask: Mask::Full
            }
            .to_string(),
            "AB"
        );
        assert_eq!(
            PatternByte {
                byte: 0xAB,
                mask: Mask::HighOnly
            }
            .to_string(),
            "A?"
        );
        assert_eq!(
            PatternByte {
                byte: 0xAB,
                mask: Mask::LowOnly
            }
            .to_string(),
            "?B"
        );
        assert_eq!(
            PatternByte {
                byte: 0xAB,
                mask: Mask::Wildcard
            }
            .to_string(),
            "??"
        );
    }

    #[test]
    fn test_from_u8() {
        assert_eq!(
            PatternByte::from(0xAB),
            PatternByte {
                byte: 0xAB,
                mask: Mask::Full
            }
        );
        assert_eq!(
            PatternByte::from(0x12),
            PatternByte {
                byte: 0x12,
                mask: Mask::Full
            }
        );
    }

    #[test]
    fn test_from_str() {
        assert_eq!(
            PatternByte::from_str("    "),
            Ok(PatternByte {
                byte: 0x00,
                mask: Mask::Full
            })
        );
        assert_eq!(
            PatternByte::from_str("  0  "),
            Ok(PatternByte {
                byte: 0x00,
                mask: Mask::Full
            })
        );
        assert_eq!(
            PatternByte::from_str("AB"),
            Ok(PatternByte {
                byte: 0xAB,
                mask: Mask::Full
            })
        );
        assert_eq!(
            PatternByte::from_str("0xA"),
            Ok(PatternByte {
                byte: 0x0A,
                mask: Mask::Full
            })
        );
        assert_eq!(
            PatternByte::from_str("0xAB"),
            Ok(PatternByte {
                byte: 0xAB,
                mask: Mask::Full
            })
        );
        assert_eq!(
            PatternByte::from_str("0X12"),
            Ok(PatternByte {
                byte: 0x12,
                mask: Mask::Full
            })
        );
        assert_eq!(
            PatternByte::from_str("0xA?"),
            Ok(PatternByte {
                byte: 0xA0,
                mask: Mask::HighOnly
            })
        );
        assert_eq!(
            PatternByte::from_str("0x?A"),
            Ok(PatternByte {
                byte: 0x0A,
                mask: Mask::LowOnly
            })
        );
        assert_eq!(
            PatternByte::from_str("0x??"),
            Ok(PatternByte {
                byte: 0x00,
                mask: Mask::Wildcard
            })
        );
        assert!(PatternByte::from_str("ABC").is_err());
        assert!(PatternByte::from_str("0xABC").is_err());
        assert!(PatternByte::from_str("X").is_err());
        assert!(PatternByte::from_str("???").is_err());
    }

    #[test]
    fn test_partial_eq_u8() {
        assert_eq!(
            PatternByte {
                byte: 0xAB,
                mask: Mask::Full
            },
            0xAB
        );
        assert_eq!(
            PatternByte {
                byte: 0xAB,
                mask: Mask::HighOnly
            },
            0xAB
        );
        assert_eq!(
            PatternByte {
                byte: 0xAB,
                mask: Mask::LowOnly
            },
            0xAB
        );
        assert_eq!(
            PatternByte {
                byte: 0xAB,
                mask: Mask::Wildcard
            },
            0xAB
        );
        assert_eq!(
            PatternByte {
                byte: 0xAB,
                mask: Mask::HighOnly
            },
            0xAA
        );
        assert_eq!(
            PatternByte {
                byte: 0xAB,
                mask: Mask::LowOnly
            },
            0xBB
        );
        assert_eq!(
            PatternByte {
                byte: 0xAB,
                mask: Mask::Wildcard
            },
            0xFF
        );
        assert_ne!(
            PatternByte {
                byte: 0xAB,
                mask: Mask::Full
            },
            0x12
        );
        assert_ne!(
            PatternByte {
                byte: 0xAB,
                mask: Mask::HighOnly
            },
            0xBA
        );
        assert_ne!(
            PatternByte {
                byte: 0xAB,
                mask: Mask::LowOnly
            },
            0xFF
        );
    }

    #[test]
    fn test_partial_ord_u8() {
        assert!(
            PatternByte {
                byte: 0xAB,
                mask: Mask::Full
            } <= 0xAB
        );
        assert!(
            PatternByte {
                byte: 0xAB,
                mask: Mask::Full
            } >= 0xAB
        );
        assert!(
            PatternByte {
                byte: 0x12,
                mask: Mask::Full
            } < 0xAB
        );
        assert!(
            PatternByte {
                byte: 0x12,
                mask: Mask::Full
            } > 0x10
        );
        assert!(
            PatternByte {
                byte: 0x12,
                mask: Mask::HighOnly
            } > 0x0F
        );
        assert!(
            PatternByte {
                byte: 0x12,
                mask: Mask::HighOnly
            } < 0xFF
        );
        assert!(
            PatternByte {
                byte: 0x12,
                mask: Mask::LowOnly
            } > 0xF0
        );
        assert!(
            PatternByte {
                byte: 0x12,
                mask: Mask::LowOnly
            } < 0xFF
        );
        assert!(
            PatternByte {
                byte: 0x12,
                mask: Mask::Wildcard
            } <= 0xFF
        );
        assert!(
            PatternByte {
                byte: 0x12,
                mask: Mask::Wildcard
            } >= 0x00
        );
    }

    #[test]
    fn test_new() {
        assert_eq!(
            PatternByte::new(0xAB, Mask::Full),
            PatternByte {
                byte: 0xAB,
                mask: Mask::Full
            }
        );
        assert_eq!(
            PatternByte::new(0x12, Mask::HighOnly),
            PatternByte {
                byte: 0x12,
                mask: Mask::HighOnly
            }
        );
        assert_eq!(
            PatternByte::new(0xCD, Mask::LowOnly),
            PatternByte {
                byte: 0xCD,
                mask: Mask::LowOnly
            }
        );
        assert_eq!(
            PatternByte::new(0xFF, Mask::Wildcard),
            PatternByte {
                byte: 0xFF,
                mask: Mask::Wildcard
            }
        );
    }

    #[test]
    #[allow(clippy::byte_char_slices)]
    fn test_try_from_u8_slice() {
        assert_eq!(
            PatternByte::try_from([b'A'].as_slice()),
            Ok(PatternByte {
                byte: 0x0A,
                mask: Mask::Full
            })
        );
        assert_eq!(
            PatternByte::try_from([b'3', b'B'].as_slice()),
            Ok(PatternByte {
                byte: 0x3B,
                mask: Mask::Full
            })
        );
        assert_eq!(
            PatternByte::try_from([b'A', b'B'].as_slice()),
            Ok(PatternByte {
                byte: 0xAB,
                mask: Mask::Full
            })
        );
        assert_eq!(
            PatternByte::try_from([b'A', b'?'].as_slice()),
            Ok(PatternByte {
                byte: 0xA0,
                mask: Mask::HighOnly
            })
        );
        assert_eq!(
            PatternByte::try_from([b'?', b'B'].as_slice()),
            Ok(PatternByte {
                byte: 0x0B,
                mask: Mask::LowOnly
            })
        );
        assert_eq!(
            PatternByte::try_from([b'?', b'?'].as_slice()),
            Ok(PatternByte {
                byte: 0x00,
                mask: Mask::Wildcard
            })
        );
        assert!(PatternByte::try_from([b'A', b'Z'].as_slice()).is_err());
        assert!(PatternByte::try_from([b'x'].as_slice()).is_err());
    }

    #[test]
    fn test_apply() {
        assert_eq!(
            PatternByte {
                byte: 0xAB,
                mask: Mask::Full
            }
            .apply(),
            0xAB
        );
        assert_eq!(
            PatternByte {
                byte: 0xAB,
                mask: Mask::HighOnly
            }
            .apply(),
            0xA0
        );
        assert_eq!(
            PatternByte {
                byte: 0xAB,
                mask: Mask::LowOnly
            }
            .apply(),
            0x0B
        );
        assert_eq!(
            PatternByte {
                byte: 0xAB,
                mask: Mask::Wildcard
            }
            .apply(),
            0x00
        );
    }

    #[test]
    fn test_merge() {
        // 0xAB merge 0xAB = 0xAB
        let sut = PatternByte {
            byte: 0xAB,
            mask: Mask::Full,
        }
        .merge(PatternByte {
            byte: 0xAB,
            mask: Mask::Full,
        });
        assert_eq!(sut.byte, 0xAB);
        assert_eq!(sut.mask, Mask::Full);

        // 0xAB merge 0xAC = 0xA?
        let sut = PatternByte {
            byte: 0xAB,
            mask: Mask::Full,
        }
        .merge(PatternByte {
            byte: 0xAC,
            mask: Mask::Full,
        });
        assert_eq!(sut.byte, 0xAB);
        assert_eq!(sut.mask, Mask::HighOnly);

        // 0xAB merge 0xBB = 0x?B
        let sut = PatternByte {
            byte: 0xAB,
            mask: Mask::Full,
        }
        .merge(PatternByte {
            byte: 0xBB,
            mask: Mask::Full,
        });
        assert_eq!(sut.byte, 0xAB);
        assert_eq!(sut.mask, Mask::LowOnly);

        // 0xAB merge 0xBA = 0x??
        let sut = PatternByte {
            byte: 0xAB,
            mask: Mask::Full,
        }
        .merge(PatternByte {
            byte: 0xBA,
            mask: Mask::Full,
        });
        assert_eq!(sut.byte, 0xAB);
        assert_eq!(sut.mask, Mask::Wildcard);

        // 0xA? merge 0xAB = 0xA?
        let sut = PatternByte {
            byte: 0xAB,
            mask: Mask::HighOnly,
        }
        .merge(PatternByte {
            byte: 0xAB,
            mask: Mask::Full,
        });
        assert_eq!(sut.byte, 0xAB);
        assert_eq!(sut.mask, Mask::HighOnly);

        // 0xA? merge 0xA? = 0xA?
        let sut = PatternByte {
            byte: 0xAB,
            mask: Mask::HighOnly,
        }
        .merge(PatternByte {
            byte: 0xAB,
            mask: Mask::HighOnly,
        });
        assert_eq!(sut.byte, 0xAB);
        assert_eq!(sut.mask, Mask::HighOnly);

        // 0xA? merge 0xB? = 0x??
        let sut = PatternByte {
            byte: 0xAB,
            mask: Mask::HighOnly,
        }
        .merge(PatternByte {
            byte: 0xBA,
            mask: Mask::HighOnly,
        });
        assert_eq!(sut.byte, 0xAB);
        assert_eq!(sut.mask, Mask::Wildcard);

        // 0xA? merge 0x?B = 0x??
        let sut = PatternByte {
            byte: 0xAB,
            mask: Mask::HighOnly,
        }
        .merge(PatternByte {
            byte: 0xAB,
            mask: Mask::LowOnly,
        });
        assert_eq!(sut.byte, 0xAB);
        assert_eq!(sut.mask, Mask::Wildcard);

        // 0xA? merge 0x?? = 0x??
        let sut = PatternByte {
            byte: 0xAB,
            mask: Mask::HighOnly,
        }
        .merge(PatternByte {
            byte: 0xAB,
            mask: Mask::Wildcard,
        });
        assert_eq!(sut.byte, 0xAB);
        assert_eq!(sut.mask, Mask::Wildcard);

        // 0xA? merge 0xBA = 0x??
        let sut = PatternByte {
            byte: 0xAB,
            mask: Mask::HighOnly,
        }
        .merge(PatternByte {
            byte: 0xBA,
            mask: Mask::Full,
        });
        assert_eq!(sut.byte, 0xAB);
        assert_eq!(sut.mask, Mask::Wildcard);

        // 0x?B merge 0xAB = 0x?B
        let sut = PatternByte {
            byte: 0xAB,
            mask: Mask::LowOnly,
        }
        .merge(PatternByte {
            byte: 0xAB,
            mask: Mask::Full,
        });
        assert_eq!(sut.byte, 0xAB);
        assert_eq!(sut.mask, Mask::LowOnly);

        // 0x?B merge 0x?B = 0x?B
        let sut = PatternByte {
            byte: 0xAB,
            mask: Mask::LowOnly,
        }
        .merge(PatternByte {
            byte: 0xAB,
            mask: Mask::LowOnly,
        });
        assert_eq!(sut.byte, 0xAB);
        assert_eq!(sut.mask, Mask::LowOnly);

        // 0x?B merge 0x?C = 0x?B
        let sut = PatternByte {
            byte: 0xAB,
            mask: Mask::LowOnly,
        }
        .merge(PatternByte {
            byte: 0xAC,
            mask: Mask::LowOnly,
        });
        assert_eq!(sut.byte, 0xAB);
        assert_eq!(sut.mask, Mask::Wildcard);

        // 0x?B merge 0xAC = 0x??
        let sut = PatternByte {
            byte: 0xAB,
            mask: Mask::LowOnly,
        }
        .merge(PatternByte {
            byte: 0xAC,
            mask: Mask::Full,
        });
        assert_eq!(sut.byte, 0xAB);
        assert_eq!(sut.mask, Mask::Wildcard);

        // 0x?? merge 0x?? = 0x??
        let sut = PatternByte {
            byte: 0xAB,
            mask: Mask::Wildcard,
        }
        .merge(PatternByte {
            byte: 0xAB,
            mask: Mask::Wildcard,
        });
        assert_eq!(sut.byte, 0xAB);
        assert_eq!(sut.mask, Mask::Wildcard);
    }
}
