use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a capped quote target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedQuoteTargetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CappedQuoteTargetStatus {
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

impl std::fmt::Display for CappedQuoteTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped quote target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedQuoteTarget {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub owner_id: String,
    pub budget: f64,
    pub status: CappedQuoteTargetStatus,
}

impl CappedQuoteTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedQuoteTargetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedQuoteTargetStatus) -> bool {
        match self.status {
            CappedQuoteTargetStatus::Draft => {
                return next == CappedQuoteTargetStatus::Active;
            }
            CappedQuoteTargetStatus::Pending => {
                return next == CappedQuoteTargetStatus::Suspended || next == CappedQuoteTargetStatus::Closed;
            }
            CappedQuoteTargetStatus::Active => {
                return next == CappedQuoteTargetStatus::Closed;
            }
            CappedQuoteTargetStatus::Suspended => {
                match next {
                    CappedQuoteTargetStatus::Closed => {
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

    pub fn move_to(&mut self, next: CappedQuoteTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped quote target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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
        self.budget
    }

    pub fn copy(&self) -> CappedQuoteTarget {
        CappedQuoteTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            owner_id: self.owner_id.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a capped quote target in status draft.
pub fn new_capped_quote_target(code: &str, label: &str) -> CappedQuoteTarget {
    CappedQuoteTarget {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        owner_id: "".to_string(),
        budget: 0.0,
        status: CappedQuoteTargetStatus::Draft,
    }
}
