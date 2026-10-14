use super::*;

use acme_kit::text;

/// Where a manual grant agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualGrantAgreementStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ManualGrantAgreementStatus {
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

impl std::fmt::Display for ManualGrantAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual grant agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualGrantAgreement {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub notes: String,
    pub priority: i64,
    pub name: String,
    pub region: String,
    pub minimum: f64,
    pub created_at: i64,
    pub status: ManualGrantAgreementStatus,
}

impl ManualGrantAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualGrantAgreementStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualGrantAgreementStatus) -> bool {
        match self.status {
            ManualGrantAgreementStatus::Open => {
                match next {
                    ManualGrantAgreementStatus::Resolved | ManualGrantAgreementStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualGrantAgreementStatus::InReview => {
                return next == ManualGrantAgreementStatus::Resolved || next == ManualGrantAgreementStatus::Archived;
            }
            ManualGrantAgreementStatus::Resolved => {
                match next {
                    ManualGrantAgreementStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualGrantAgreementStatus::Reopened => {
                match next {
                    ManualGrantAgreementStatus::Archived => {
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

    pub fn move_to(&mut self, next: ManualGrantAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual grant agreement {} cannot move from {} to {}",
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
        format!("{} {}", "manual grant agreement", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.minimum * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualGrantAgreement {
        ManualGrantAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            notes: self.notes.clone(),
            priority: self.priority,
            name: self.name.clone(),
            region: self.region.clone(),
            minimum: self.minimum,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a manual grant agreement in status open.
pub fn new_manual_grant_agreement(code: &str, title: &str) -> ManualGrantAgreement {
    ManualGrantAgreement {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        notes: "".to_string(),
        priority: 0,
        name: "".to_string(),
        region: "".to_string(),
        minimum: 0.0,
        created_at: 0,
        status: ManualGrantAgreementStatus::Open,
    }
}
