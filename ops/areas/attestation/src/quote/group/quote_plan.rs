use super::*;

use acme_kit::text;

/// Where a quote plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuotePlanStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl QuotePlanStatus {
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

impl std::fmt::Display for QuotePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A quote plan as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct QuotePlan {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub grace_days: i64,
    pub external_ref: String,
    pub tags: Vec<String>,
    pub summary: String,
    pub currency: String,
    pub status: QuotePlanStatus,
}

impl QuotePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuotePlanStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuotePlanStatus) -> bool {
        match self.status {
            QuotePlanStatus::Open => {
                match next {
                    QuotePlanStatus::InReview | QuotePlanStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuotePlanStatus::InReview => {
                return next == QuotePlanStatus::Resolved;
            }
            QuotePlanStatus::Resolved => {
                match next {
                    QuotePlanStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuotePlanStatus::Reopened => {
                return next == QuotePlanStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QuotePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "quote plan", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> QuotePlan {
        QuotePlan {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            grace_days: self.grace_days,
            external_ref: self.external_ref.clone(),
            tags: self.tags.clone(),
            summary: self.summary.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a quote plan in status open.
pub fn new_quote_plan(code: &str, name: &str) -> QuotePlan {
    QuotePlan {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        grace_days: 0,
        external_ref: "".to_string(),
        tags: Vec::new(),
        summary: "".to_string(),
        currency: "".to_string(),
        status: QuotePlanStatus::Open,
    }
}
