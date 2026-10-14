use super::*;

use acme_kit::text;

/// Where a lenient rule draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientRuleDraftStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientRuleDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for LenientRuleDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient rule draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleDraft {
    pub code: String,
    pub label: String,
    pub owner_id: String,
    pub priority: i64,
    pub title: String,
    pub unit: String,
    pub duration_minutes: i64,
    pub issuer: String,
    pub cost: f64,
    pub capacity: i64,
    pub status: LenientRuleDraftStatus,
}

impl LenientRuleDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientRuleDraftStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientRuleDraftStatus) -> bool {
        match self.status {
            LenientRuleDraftStatus::Open => {
                match next {
                    LenientRuleDraftStatus::Resolved | LenientRuleDraftStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRuleDraftStatus::InReview => {
                match next {
                    LenientRuleDraftStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRuleDraftStatus::Resolved => {
                match next {
                    LenientRuleDraftStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRuleDraftStatus::Reopened => {
                match next {
                    LenientRuleDraftStatus::Archived => {
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

    pub fn move_to(&mut self, next: LenientRuleDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient rule draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.priority as f64 - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientRuleDraft {
        LenientRuleDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            owner_id: self.owner_id.clone(),
            priority: self.priority,
            title: self.title.clone(),
            unit: self.unit.clone(),
            duration_minutes: self.duration_minutes,
            issuer: self.issuer.clone(),
            cost: self.cost,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a lenient rule draft in status open.
pub fn new_lenient_rule_draft(code: &str, label: &str) -> LenientRuleDraft {
    LenientRuleDraft {
        code: code.to_string(),
        label: label.to_string(),
        owner_id: "".to_string(),
        priority: 0,
        title: "".to_string(),
        unit: "".to_string(),
        duration_minutes: 0,
        issuer: "".to_string(),
        cost: 0.0,
        capacity: 0,
        status: LenientRuleDraftStatus::Open,
    }
}
