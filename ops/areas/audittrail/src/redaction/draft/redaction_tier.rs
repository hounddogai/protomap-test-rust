use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a redaction tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionTierStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RedactionTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RedactionTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One redaction tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionTier {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub capacity: i64,
    pub threshold: f64,
    pub version: i64,
    pub attributes: BTreeMap<String, String>,
    pub discount: f64,
    pub status: RedactionTierStatus,
}

impl RedactionTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionTierStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionTierStatus) -> bool {
        match self.status {
            RedactionTierStatus::Created => {
                return next == RedactionTierStatus::Validated || next == RedactionTierStatus::Published;
            }
            RedactionTierStatus::Validated => {
                match next {
                    RedactionTierStatus::Published | RedactionTierStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionTierStatus::Published => {
                match next {
                    RedactionTierStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RedactionTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.capacity as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionTier {
        RedactionTier {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            capacity: self.capacity,
            threshold: self.threshold,
            version: self.version,
            attributes: self.attributes.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a redaction tier in status created.
pub fn new_redaction_tier(code: &str, name: &str) -> RedactionTier {
    RedactionTier {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        capacity: 0,
        threshold: 0.0,
        version: 0,
        attributes: BTreeMap::new(),
        discount: 0.0,
        status: RedactionTierStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_redaction_tier("code-648", "name-136");
        assert_eq!(item.status, RedactionTierStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_redaction_tier("code-648", "name-136");
        if item.move_to(RedactionTierStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionTierStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_redaction_tier("code-648", "name-136");
        item.status = RedactionTierStatus::Retired;
        assert!(
            item.move_to(RedactionTierStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
