use super::*;

use acme_kit::text;

/// Where a retention bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionBundleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RetentionBundleStatus {
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

impl std::fmt::Display for RetentionBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A retention bundle as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionBundle {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub category: String,
    pub owner_id: String,
    pub amount: f64,
    pub host: String,
    pub status: RetentionBundleStatus,
}

impl RetentionBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionBundleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionBundleStatus) -> bool {
        match self.status {
            RetentionBundleStatus::Open => {
                return next == RetentionBundleStatus::InReview || next == RetentionBundleStatus::Reopened;
            }
            RetentionBundleStatus::InReview => {
                match next {
                    RetentionBundleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionBundleStatus::Resolved => {
                return next == RetentionBundleStatus::Reopened || next == RetentionBundleStatus::Archived;
            }
            RetentionBundleStatus::Reopened => {
                match next {
                    RetentionBundleStatus::Archived => {
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

    pub fn move_to(&mut self, next: RetentionBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention bundle {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.created_at as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionBundle {
        RetentionBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            category: self.category.clone(),
            owner_id: self.owner_id.clone(),
            amount: self.amount,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a retention bundle in status open.
pub fn new_retention_bundle(code: &str, label: &str) -> RetentionBundle {
    RetentionBundle {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        category: "".to_string(),
        owner_id: "".to_string(),
        amount: 0.0,
        host: "".to_string(),
        status: RetentionBundleStatus::Open,
    }
}
