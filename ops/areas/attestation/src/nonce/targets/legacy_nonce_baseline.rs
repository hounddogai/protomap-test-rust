use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a legacy nonce baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyNonceBaselineStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LegacyNonceBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LegacyNonceBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy nonce baseline as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyNonceBaseline {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub priority: i64,
    pub revision: i64,
    pub unit: String,
    pub summary: String,
    pub cost: f64,
    pub attributes: BTreeMap<String, String>,
    pub status: LegacyNonceBaselineStatus,
}

impl LegacyNonceBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyNonceBaselineStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyNonceBaselineStatus) -> bool {
        match self.status {
            LegacyNonceBaselineStatus::Proposed => {
                match next {
                    LegacyNonceBaselineStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyNonceBaselineStatus::Approved => {
                match next {
                    LegacyNonceBaselineStatus::Rejected | LegacyNonceBaselineStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyNonceBaselineStatus::Rejected => {
                match next {
                    LegacyNonceBaselineStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: LegacyNonceBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy nonce baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy nonce baseline", self.code)
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

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.revision as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyNonceBaseline {
        LegacyNonceBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            priority: self.priority,
            revision: self.revision,
            unit: self.unit.clone(),
            summary: self.summary.clone(),
            cost: self.cost,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a legacy nonce baseline in status proposed.
pub fn new_legacy_nonce_baseline(code: &str, title: &str) -> LegacyNonceBaseline {
    LegacyNonceBaseline {
        code: code.to_string(),
        title: title.to_string(),
        fingerprint: "".to_string(),
        priority: 0,
        revision: 0,
        unit: "".to_string(),
        summary: "".to_string(),
        cost: 0.0,
        attributes: BTreeMap::new(),
        status: LegacyNonceBaselineStatus::Proposed,
    }
}
