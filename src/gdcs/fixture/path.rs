use std::{fmt, str};

use crate::gdcs::GdcsError;
use crate::gdcs::fixture::FixtureId;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FixturePath {
    ids: [FixtureId; Self::MAX_LEN],
    len: u8,
}

impl FixturePath {
    const MAX_LEN: usize = 8;

    pub fn new(root_id: FixtureId) -> Self {
        let mut ids = [FixtureId::new(1).unwrap(); Self::MAX_LEN];
        ids[0] = root_id;
        FixturePath { ids, len: 1 }
    }

    pub fn push(&mut self, id: FixtureId) {
        let len = self.len();
        assert!(len < Self::MAX_LEN, "FixturePath capacity exceeded (max {})", Self::MAX_LEN);
        self.ids[len] = id;
        self.len = (len + 1) as u8;
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_root_fixture(&self) -> bool {
        self.len == 1
    }

    pub fn sub_len(&self) -> usize {
        assert!(self.len() >= 1, "FixturePath must have at least a root");
        self.len() - 1
    }

    pub fn root(&self) -> FixtureId {
        self.ids[0]
    }

    pub fn last(&self) -> FixtureId {
        let l = self.len();
        assert!(l >= 1, "FixturePath must have at least a root");
        self.ids[l - 1]
    }

    pub fn as_slice(&self) -> &[FixtureId] {
        &self.ids[..self.len()]
    }

    pub fn iter(&self) -> std::slice::Iter<'_, FixtureId> {
        self.as_slice().iter()
    }

    pub fn replace_last(&mut self, sub_id: FixtureId) {
        let l = self.len();
        assert!(l >= 1, "FixturePath must have at least a root");
        self.ids[l - 1] = sub_id;
    }

    pub fn extended_with(mut self, part: FixtureId) -> FixturePath {
        self.push(part);
        self
    }
}

impl AsRef<[FixtureId]> for FixturePath {
    fn as_ref(&self) -> &[FixtureId] {
        self.as_slice()
    }
}

impl From<FixtureId> for FixturePath {
    fn from(id: FixtureId) -> Self {
        FixturePath::new(id)
    }
}

impl From<&[FixtureId]> for FixturePath {
    fn from(slice: &[FixtureId]) -> Self {
        assert!(
            slice.len() <= FixturePath::MAX_LEN,
            "FixturePath slice length {} exceeds capacity {}",
            slice.len(),
            FixturePath::MAX_LEN
        );
        let mut ids = [FixtureId::new(1).unwrap(); FixturePath::MAX_LEN];
        for (i, v) in slice.iter().enumerate() {
            ids[i] = *v;
        }
        FixturePath { ids, len: slice.len() as u8 }
    }
}

impl IntoIterator for FixturePath {
    type Item = FixtureId;
    type IntoIter = std::vec::IntoIter<FixtureId>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().to_vec().into_iter()
    }
}

impl<'a> IntoIterator for &'a FixturePath {
    type Item = &'a FixtureId;
    type IntoIter = std::slice::Iter<'a, FixtureId>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl std::cmp::PartialOrd for FixturePath {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for FixturePath {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let a = self.as_slice();
        let b = other.as_slice();
        for (x, y) in a.iter().zip(b.iter()) {
            let ord = x.cmp(y);
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
        }
        a.len().cmp(&b.len())
    }
}
impl fmt::Display for FixturePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for id in self.as_slice() {
            if !first {
                write!(f, ".")?;
            }
            write!(f, "{}", id)?;
            first = false;
        }
        Ok(())
    }
}

impl fmt::Debug for FixturePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FixturePath(")?;
        fmt::Display::fmt(self, f)?;
        write!(f, ")")
    }
}

impl str::FromStr for FixturePath {
    type Err = GdcsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();

        if parts.is_empty() {
            return Err(GdcsError::FailedToParseFixturePath {
                message: "FixturePath string is empty".to_string(),
            });
        }

        if parts.len() > FixturePath::MAX_LEN {
            return Err(GdcsError::FailedToParseFixturePath {
                message: format!(
                    "FixturePath string has too many parts (max {})",
                    FixturePath::MAX_LEN
                ),
            });
        }
        let mut ids = [FixtureId::new(1).unwrap(); FixturePath::MAX_LEN];
        for (i, part) in parts.iter().enumerate() {
            ids[i] = FixtureId::from_str(part)?;
        }
        Ok(FixturePath { ids, len: parts.len() as u8 })
    }
}

impl serde::Serialize for FixturePath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use std::fmt::Write;
        let mut s = String::new();
        write!(&mut s, "{}", self).unwrap();
        serializer.serialize_str(&s)
    }
}

impl<'de> serde::Deserialize<'de> for FixturePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct FixturePathVisitor;

        impl<'de> serde::de::Visitor<'de> for FixturePathVisitor {
            type Value = FixturePath;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing a FixturePath")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::str::FromStr;
                FixturePath::from_str(v).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(FixturePathVisitor)
    }
}

#[macro_export]
macro_rules! fpath {
    ( $first:expr $(, $rest:expr )* $(,)? ) => {{
        let mut p = $crate::fixture::FixturePath::new($crate::fixture::FixtureId::new($first).unwrap());
        $( p.push($crate::fixture::FixtureId::new($rest).unwrap()); )*
        p
    }};
    ( $first:expr $(, $rest:expr )* $(,)? ) => {{
        let mut p = $crate::fixture::FixturePath::new($first);
        $( p.push($rest); )*
        p
    }};
}
