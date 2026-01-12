use crate::Identifier;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct Trigger {
    id: Identifier,
    value: TriggerValue,
}

impl Trigger {
    pub fn new(id: Identifier, value: TriggerValue) -> Self {
        Self { id: id.into(), value }
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }

    pub fn value(&self) -> &TriggerValue {
        &self.value
    }
}

impl Display for Trigger {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.value {
            TriggerValue::Empty => write!(f, "{}", &self.id),
            _ => write!(f, "{}={}", &self.id, &self.value),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum TriggerValue {
    Empty,
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

impl Display for TriggerValue {
    /// Render values in a straightforward, parseable textual form and prefix
    /// each output with a short type indicator (single-letter):
    /// - Empty -> "e:" (no value after the colon)
    /// - Integer -> "i:<decimal>", e.g. "i:42"
    /// - Float -> "f:<decimal>", e.g. "f:3.14"
    /// - Boolean -> "b:true" or "b:false"
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TriggerValue::Empty => write!(f, "e:"),
            TriggerValue::Integer(i) => write!(f, "i:{}", i),
            TriggerValue::Float(fl) => {
                // Use the default formatting for f64; that's suitable for round-trip parsing
                write!(f, "f:{}", fl)
            }
            TriggerValue::Boolean(b) => write!(f, "b:{}", b),
        }
    }
}

impl FromStr for TriggerValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Accept "e:" for empty, otherwise expect "<prefix>:<rest>"
        if s == "e:" {
            return Ok(TriggerValue::Empty);
        }
        let mut parts = s.splitn(2, ':');
        let prefix = parts.next().ok_or_else(|| "missing prefix".to_string())?;
        let rest = parts.next().ok_or_else(|| "missing ':' after prefix".to_string())?;
        match prefix {
            "i" => {
                let i = rest.parse::<i64>().map_err(|e| e.to_string())?;
                Ok(TriggerValue::Integer(i))
            }
            "f" => {
                let fl = rest.parse::<f64>().map_err(|e| e.to_string())?;
                Ok(TriggerValue::Float(fl))
            }
            "b" => {
                let b = rest.parse::<bool>().map_err(|e| e.to_string())?;
                Ok(TriggerValue::Boolean(b))
            }
            "e" => {
                // Allow "e:" with empty rest as well
                if rest.is_empty() {
                    Ok(TriggerValue::Empty)
                } else {
                    Err("unexpected data after empty prefix".to_string())
                }
            }
            other => Err(format!("unknown prefix '{}'", other)),
        }
    }
}

impl FromStr for Trigger {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Two forms:
        // - "ident"  => empty value
        // - "ident=<value>" where <value> is a TriggerValue textual form
        if let Some(eq_pos) = s.find('=') {
            let (id_str, val_str) = s.split_at(eq_pos);
            let val_str = &val_str[1..]; // skip '='
            let id = Identifier::from_str(id_str).map_err(|e| e.to_string())?;
            let value = TriggerValue::from_str(val_str)?;
            Ok(Trigger::new(id, value))
        } else {
            let id = Identifier::from_str(s).map_err(|e| e.to_string())?;
            Ok(Trigger::new(id, TriggerValue::Empty))
        }
    }
}
