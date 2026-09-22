use crate::byte::{Mask, PatternByte};

/// Represents a sequence of masked bytes.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pattern {
    bytes: Vec<PatternByte>,
}

impl std::ops::Deref for Pattern {
    type Target = [PatternByte];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.bytes.iter();
        if let Some(first) = iter.next() {
            write!(f, "{first}")?;
            for p in iter {
                write!(f, " {p}")?;
            }
        }
        Ok(())
    }
}

impl FromIterator<PatternByte> for Pattern {
    #[inline]
    fn from_iter<T: IntoIterator<Item = PatternByte>>(iter: T) -> Self {
        Self {
            bytes: iter.into_iter().collect(),
        }
    }
}

impl FromIterator<u8> for Pattern {
    #[inline]
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        Self {
            bytes: iter.into_iter().map(PatternByte::from).collect(),
        }
    }
}

impl std::str::FromStr for Pattern {
    type Err = crate::byte::InvalidByteStringError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        let bytes = trimmed
            .strip_prefix("0x")
            .or_else(|| trimmed.strip_prefix("0X"))
            .unwrap_or(trimmed)
            .bytes()
            .filter(|&b| b != b' ')
            .collect::<Vec<_>>();

        if bytes.is_empty() {
            return Ok(Self::default());
        }
        let patterns = bytes.chunks(2).map(PatternByte::try_from).collect::<Result<Vec<_>, _>>()?;
        Ok(Self { bytes: patterns })
    }
}

impl IntoIterator for Pattern {
    type Item = PatternByte;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.bytes.into_iter()
    }
}

impl<T: AsRef<[u8]>> PartialEq<T> for Pattern {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.bytes.iter().eq(other.as_ref().iter())
    }
}

impl Pattern {
    /// Merges this `Pattern` into a broader pattern.
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        let (longer, shorter) = if self.bytes.len() > other.bytes.len() {
            (&self.bytes, &other.bytes)
        } else {
            (&other.bytes, &self.bytes)
        };

        let leftovers = longer[shorter.len()..]
            .iter()
            .map(|x| PatternByte::new(x.byte, Mask::Wildcard));
        let patterns = self
            .bytes
            .iter()
            .zip(other.bytes.iter())
            .map(|(a, b)| a.merge(*b))
            .chain(leftovers)
            .collect::<Vec<_>>();
        Self { bytes: patterns }
    }

    /// Finds the position of this pattern in the given data.
    #[inline]
    pub fn find(&self, data: impl AsRef<[u8]>) -> Option<usize> {
        if self.bytes.is_empty() {
            Some(0)
        } else {
            data.as_ref().windows(self.bytes.len()).position(|w| *self == w)
        }
    }
}

#[cfg(test)]
#[allow(clippy::too_many_lines)]
mod pattern_tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_display() {
        let full = PatternByte::new(0xAB, Mask::Full);
        let high = PatternByte::new(0xAB, Mask::HighOnly);
        let low = PatternByte::new(0xAB, Mask::LowOnly);
        let wildcard = PatternByte::new(0xAB, Mask::Wildcard);
        assert_eq!(Pattern::default().to_string(), "");
        assert_eq!(
            Pattern {
                bytes: vec![full, high, low, wildcard]
            }
            .to_string(),
            "AB A? ?B ??"
        );
    }

    #[test]
    fn test_from_iter_pattern() {
        let full = PatternByte::new(0xAB, Mask::Full);
        let high = PatternByte::new(0xAB, Mask::HighOnly);
        let low = PatternByte::new(0xAB, Mask::LowOnly);
        let wildcard = PatternByte::new(0xAB, Mask::Wildcard);
        assert_eq!(Pattern::from_iter(Vec::<PatternByte>::new()), Pattern { bytes: vec![] });
        assert_eq!(Pattern::from_iter(vec![full, full]), Pattern { bytes: vec![full, full] });
        assert_eq!(
            Pattern::from_iter(vec![full, high, low, wildcard]),
            Pattern {
                bytes: vec![full, high, low, wildcard]
            }
        );
    }

    #[test]
    fn test_from_iter_u8() {
        let p1 = PatternByte::new(0xAB, Mask::Full);
        let p2 = PatternByte::new(0x12, Mask::Full);
        let p3 = PatternByte::new(0xFF, Mask::Full);
        assert_eq!(Pattern::from_iter(Vec::<u8>::new()), Pattern { bytes: vec![] });
        assert_eq!(Pattern::from_iter(vec![p1.byte, p2.byte]), Pattern { bytes: vec![p1, p2] });
        assert_eq!(
            Pattern::from_iter(vec![p1.byte, p3.byte, p2.byte]),
            Pattern { bytes: vec![p1, p3, p2] }
        );
    }

    #[test]
    fn test_from_str() {
        let full = PatternByte::new(0xAB, Mask::Full);
        let high = PatternByte::new(0xA0, Mask::HighOnly);
        let low = PatternByte::new(0x0B, Mask::LowOnly);
        let wildcard = PatternByte::new(0x00, Mask::Wildcard);
        assert_eq!(Pattern::from_str("0xABA?"), Ok(Pattern { bytes: vec![full, high] }));
        assert_eq!(
            Pattern::from_str("0XABA??B"),
            Ok(Pattern {
                bytes: vec![full, high, low]
            })
        );
        assert_eq!(
            Pattern::from_str("  ?? ?B A? "),
            Ok(Pattern {
                bytes: vec![wildcard, low, high]
            })
        );
        assert_eq!(Pattern::from_str(""), Ok(Pattern::default()));
        assert_eq!(Pattern::from_str("0x"), Ok(Pattern::default()));
        assert_eq!(Pattern::from_str("  0X"), Ok(Pattern::default()));
        assert!(Pattern::from_str("Z").is_err());
    }

    #[test]
    fn test_into_iter() {
        let p1 = PatternByte::new(0xAB, Mask::Full);
        let p2 = PatternByte::new(0x12, Mask::Full);
        let p3 = PatternByte::new(0xFF, Mask::Full);
        assert_eq!(Pattern::default().into_iter().collect::<Vec<_>>(), Vec::<PatternByte>::new());
        assert_eq!(Pattern { bytes: vec![p1] }.into_iter().collect::<Vec<_>>(), vec![p1]);
        assert_eq!(
            Pattern { bytes: vec![p1, p2, p3] }.into_iter().collect::<Vec<_>>(),
            vec![p1, p2, p3]
        );
    }

    #[test]
    fn test_partial_eq_u8_slice() {
        let full = PatternByte::new(0xAB, Mask::Full);
        let high = PatternByte::new(0x12, Mask::HighOnly);
        let low = PatternByte::new(0xFF, Mask::LowOnly);
        let wildcard = PatternByte::new(0xCD, Mask::Wildcard);
        let pattern = Pattern {
            bytes: vec![full, high, low, wildcard],
        };
        assert!(pattern.eq(&vec![0xAB, 0x12, 0xFF, 0xCD]));
        assert!(pattern.eq(&vec![0xAB, 0x1F, 0xFF, 0xCD]));
        assert!(pattern.eq(&vec![0xAB, 0x12, 0x3F, 0xCD]));
        assert!(pattern.eq(&vec![0xAB, 0x12, 0xFF, 0xEF]));
        assert!(!pattern.eq(&vec![0xCD, 0x12, 0xFF, 0xCD]));
        assert!(!pattern.eq(&vec![0xAB, 0xF2, 0xFF, 0xCD]));
        assert!(!pattern.eq(&vec![0xAB, 0x12, 0xF2, 0xCD]));
    }

    #[test]
    fn test_merge() {
        let full = PatternByte::new(0xAB, Mask::Full);
        let high = PatternByte::new(0x12, Mask::HighOnly);
        let low = PatternByte::new(0xFF, Mask::LowOnly);
        let wildcard = PatternByte::new(0x12, Mask::Wildcard);
        let sut = Pattern {
            bytes: vec![full, high, low, wildcard],
        };
        assert_eq!(
            sut.merge(&Pattern::default()),
            Pattern {
                bytes: vec![
                    PatternByte::new(0xAB, Mask::Wildcard),
                    PatternByte::new(0x12, Mask::Wildcard),
                    PatternByte::new(0xFF, Mask::Wildcard),
                    PatternByte::new(0x12, Mask::Wildcard)
                ]
            }
        );
        assert_eq!(
            sut.merge(&Pattern {
                bytes: vec![full, high, low, wildcard]
            }),
            Pattern {
                bytes: vec![full, high, low, wildcard]
            }
        );
        assert_eq!(
            sut.merge(&Pattern { bytes: vec![full] }),
            Pattern {
                bytes: vec![
                    full,
                    PatternByte::new(0x12, Mask::Wildcard),
                    PatternByte::new(0xFF, Mask::Wildcard),
                    PatternByte::new(0x12, Mask::Wildcard)
                ]
            }
        );
        assert_eq!(
            sut.merge(&Pattern {
                bytes: vec![full, wildcard, wildcard]
            }),
            Pattern {
                bytes: vec![
                    full,
                    PatternByte::new(0x12, Mask::Wildcard),
                    PatternByte::new(0xFF, Mask::Wildcard),
                    PatternByte::new(0x12, Mask::Wildcard)
                ]
            }
        );
    }

    #[test]
    fn test_find() {
        let data = vec![0xAB, 0xCD, 0xEF];
        assert_eq!(Pattern::default().find(&data), Some(0));

        let pattern = Pattern::from_str("0xABCD").unwrap();
        assert_eq!(pattern.find(&data), Some(0));

        let pattern = Pattern::from_str("0xCDEF").unwrap();
        assert_eq!(pattern.find(&data), Some(1));

        let pattern = Pattern::from_str("0xEF").unwrap();
        assert_eq!(pattern.find(&data), Some(2));

        let pattern = Pattern::from_str("0x12").unwrap();
        assert_eq!(pattern.find(&data), None);

        let pattern = Pattern::from_str("0xABCDEF12").unwrap();
        assert_eq!(pattern.find(&data), None);

        let pattern = Pattern::from_str("0xA?").unwrap();
        assert_eq!(pattern.find(&data), Some(0));

        let pattern = Pattern::from_str("0x?BC?").unwrap();
        assert_eq!(pattern.find(&data), Some(0));

        let pattern = Pattern::from_str("0x?D??").unwrap();
        assert_eq!(pattern.find(&data), Some(1));

        let pattern = Pattern::from_str("0x?D?1").unwrap();
        assert_eq!(pattern.find(&data), None);
    }
}
