use super::*;

use acme_kit::text;

/// Where a rule entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleEntryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RuleEntryStatus {
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

impl std::fmt::Display for RuleEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleEntry {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub archived: bool,
    pub duration_minutes: i64,
    pub port: i64,
    pub ceiling: f64,
    pub revision: i64,
    pub attempts: i64,
    pub name: String,
    pub expires_at: i64,
    pub status: RuleEntryStatus,
}

impl RuleEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleEntryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleEntryStatus) -> bool {
        match self.status {
            RuleEntryStatus::Open => {
                return next == RuleEntryStatus::Archived;
            }
            RuleEntryStatus::InReview => {
                match next {
                    RuleEntryStatus::Reopened | RuleEntryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleEntryStatus::Resolved => {
                return next == RuleEntryStatus::Reopened || next == RuleEntryStatus::Archived;
            }
            RuleEntryStatus::Reopened => {
                match next {
                    RuleEntryStatus::Archived => {
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

    pub fn move_to(&mut self, next: RuleEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.grace_days as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleEntry {
        RuleEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            archived: self.archived,
            duration_minutes: self.duration_minutes,
            port: self.port,
            ceiling: self.ceiling,
            revision: self.revision,
            attempts: self.attempts,
            name: self.name.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rule entry in status open.
pub fn new_rule_entry(code: &str, label: &str) -> RuleEntry {
    RuleEntry {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        archived: false,
        duration_minutes: 0,
        port: 0,
        ceiling: 0.0,
        revision: 0,
        attempts: 0,
        name: "".to_string(),
        expires_at: 0,
        status: RuleEntryStatus::Open,
    }
}
