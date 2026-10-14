use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a redaction cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionCycleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RedactionCycleStatus {
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

impl std::fmt::Display for RedactionCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionCycle {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub pinned: bool,
    pub issuer: String,
    pub budget: f64,
    pub created_at: i64,
    pub status: RedactionCycleStatus,
}

impl RedactionCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionCycleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionCycleStatus) -> bool {
        match self.status {
            RedactionCycleStatus::Draft => {
                return next == RedactionCycleStatus::Active;
            }
            RedactionCycleStatus::Pending => {
                match next {
                    RedactionCycleStatus::Active | RedactionCycleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionCycleStatus::Active => {
                match next {
                    RedactionCycleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionCycleStatus::Suspended => {
                return next == RedactionCycleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RedactionCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "redaction cycle", self.code)
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

    pub fn effective_total(&self) -> f64 {
        ((self.created_at as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionCycle {
        RedactionCycle {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            pinned: self.pinned,
            issuer: self.issuer.clone(),
            budget: self.budget,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a redaction cycle in status draft.
pub fn new_redaction_cycle(code: &str, title: &str) -> RedactionCycle {
    RedactionCycle {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        pinned: false,
        issuer: "".to_string(),
        budget: 0.0,
        created_at: 0,
        status: RedactionCycleStatus::Draft,
    }
}
