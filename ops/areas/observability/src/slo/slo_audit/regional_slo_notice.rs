use super::*;

use acme_kit::text;

/// Where a regional slo notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalSloNoticeStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalSloNoticeStatus {
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

impl std::fmt::Display for RegionalSloNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional slo notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSloNotice {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub reviewer: String,
    pub archived: bool,
    pub rotation_days: i64,
    pub capacity: i64,
    pub category: String,
    pub cost: f64,
    pub key_id: String,
    pub retries: i64,
    pub status: RegionalSloNoticeStatus,
}

impl RegionalSloNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalSloNoticeStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalSloNoticeStatus) -> bool {
        match self.status {
            RegionalSloNoticeStatus::Open => {
                return next == RegionalSloNoticeStatus::InReview || next == RegionalSloNoticeStatus::Reopened;
            }
            RegionalSloNoticeStatus::InReview => {
                return next == RegionalSloNoticeStatus::Resolved || next == RegionalSloNoticeStatus::Archived;
            }
            RegionalSloNoticeStatus::Resolved => {
                return next == RegionalSloNoticeStatus::Reopened || next == RegionalSloNoticeStatus::Archived;
            }
            RegionalSloNoticeStatus::Reopened => {
                match next {
                    RegionalSloNoticeStatus::Archived => {
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

    pub fn move_to(&mut self, next: RegionalSloNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional slo notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional slo notice", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.updated_at as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalSloNotice {
        RegionalSloNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            reviewer: self.reviewer.clone(),
            archived: self.archived,
            rotation_days: self.rotation_days,
            capacity: self.capacity,
            category: self.category.clone(),
            cost: self.cost,
            key_id: self.key_id.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a regional slo notice in status open.
pub fn new_regional_slo_notice(code: &str, title: &str) -> RegionalSloNotice {
    RegionalSloNotice {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        reviewer: "".to_string(),
        archived: false,
        rotation_days: 0,
        capacity: 0,
        category: "".to_string(),
        cost: 0.0,
        key_id: "".to_string(),
        retries: 0,
        status: RegionalSloNoticeStatus::Open,
    }
}
