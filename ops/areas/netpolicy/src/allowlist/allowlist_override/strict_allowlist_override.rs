use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a strict allowlist override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictAllowlistOverrideStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictAllowlistOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for StrictAllowlistOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict allowlist override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictAllowlistOverride {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub attributes: BTreeMap<String, String>,
    pub cipher: String,
    pub summary: String,
    pub ceiling: f64,
    pub status: StrictAllowlistOverrideStatus,
}

impl StrictAllowlistOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictAllowlistOverrideStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictAllowlistOverrideStatus) -> bool {
        match self.status {
            StrictAllowlistOverrideStatus::Draft => {
                match next {
                    StrictAllowlistOverrideStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictAllowlistOverrideStatus::Pending => {
                match next {
                    StrictAllowlistOverrideStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictAllowlistOverrideStatus::Active => {
                match next {
                    StrictAllowlistOverrideStatus::Suspended | StrictAllowlistOverrideStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictAllowlistOverrideStatus::Suspended => {
                match next {
                    StrictAllowlistOverrideStatus::Closed => {
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

    pub fn move_to(&mut self, next: StrictAllowlistOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict allowlist override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
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

    pub fn measured_value(&self) -> f64 {
        self.ceiling
    }

    pub fn copy(&self) -> StrictAllowlistOverride {
        StrictAllowlistOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            unit: self.unit.clone(),
            attributes: self.attributes.clone(),
            cipher: self.cipher.clone(),
            summary: self.summary.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a strict allowlist override in status draft.
pub fn new_strict_allowlist_override(code: &str, name: &str) -> StrictAllowlistOverride {
    StrictAllowlistOverride {
        code: code.to_string(),
        name: name.to_string(),
        unit: "".to_string(),
        attributes: BTreeMap::new(),
        cipher: "".to_string(),
        summary: "".to_string(),
        ceiling: 0.0,
        status: StrictAllowlistOverrideStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_strict_allowlist_override("code-73", "name-801");
        assert_eq!(item.status, StrictAllowlistOverrideStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_strict_allowlist_override("code-73", "name-801");
        if item.move_to(StrictAllowlistOverrideStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictAllowlistOverrideStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_strict_allowlist_override("code-73", "name-801");
        item.status = StrictAllowlistOverrideStatus::Closed;
        assert!(
            item.move_to(StrictAllowlistOverrideStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
