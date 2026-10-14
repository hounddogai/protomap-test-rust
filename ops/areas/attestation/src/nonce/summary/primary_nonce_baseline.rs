use super::*;

use acme_kit::text;

/// Where a primary nonce baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryNonceBaselineStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PrimaryNonceBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for PrimaryNonceBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary nonce baseline as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryNonceBaseline {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub grace_days: i64,
    pub phone: String,
    pub capacity: i64,
    pub budget: f64,
    pub archived: bool,
    pub status: PrimaryNonceBaselineStatus,
}

impl PrimaryNonceBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryNonceBaselineStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryNonceBaselineStatus) -> bool {
        match self.status {
            PrimaryNonceBaselineStatus::New => {
                return next == PrimaryNonceBaselineStatus::Running || next == PrimaryNonceBaselineStatus::Done;
            }
            PrimaryNonceBaselineStatus::Queued => {
                match next {
                    PrimaryNonceBaselineStatus::Running | PrimaryNonceBaselineStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryNonceBaselineStatus::Running => {
                return next == PrimaryNonceBaselineStatus::Done || next == PrimaryNonceBaselineStatus::Failed;
            }
            PrimaryNonceBaselineStatus::Done => {
                match next {
                    PrimaryNonceBaselineStatus::Failed => {
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

    pub fn move_to(&mut self, next: PrimaryNonceBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary nonce baseline {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        (self.budget * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryNonceBaseline {
        PrimaryNonceBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            grace_days: self.grace_days,
            phone: self.phone.clone(),
            capacity: self.capacity,
            budget: self.budget,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary nonce baseline in status new.
pub fn new_primary_nonce_baseline(code: &str, label: &str) -> PrimaryNonceBaseline {
    PrimaryNonceBaseline {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        grace_days: 0,
        phone: "".to_string(),
        capacity: 0,
        budget: 0.0,
        archived: false,
        status: PrimaryNonceBaselineStatus::New,
    }
}
