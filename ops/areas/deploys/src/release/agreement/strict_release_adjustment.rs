use super::*;

use acme_kit::text;

/// Where a strict release adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictReleaseAdjustmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictReleaseAdjustmentStatus {
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

impl std::fmt::Display for StrictReleaseAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict release adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleaseAdjustment {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub visible: bool,
    pub grace_days: i64,
    pub key_id: String,
    pub owner_id: String,
    pub issuer: String,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub updated_at: i64,
    pub status: StrictReleaseAdjustmentStatus,
}

impl StrictReleaseAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictReleaseAdjustmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictReleaseAdjustmentStatus) -> bool {
        match self.status {
            StrictReleaseAdjustmentStatus::Open => {
                return next == StrictReleaseAdjustmentStatus::Archived;
            }
            StrictReleaseAdjustmentStatus::InReview => {
                match next {
                    StrictReleaseAdjustmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictReleaseAdjustmentStatus::Resolved => {
                match next {
                    StrictReleaseAdjustmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictReleaseAdjustmentStatus::Reopened => {
                return next == StrictReleaseAdjustmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictReleaseAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict release adjustment {} cannot move from {} to {}",
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
        format!("{} {}", "strict release adjustment", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.grace_days as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictReleaseAdjustment {
        StrictReleaseAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            visible: self.visible,
            grace_days: self.grace_days,
            key_id: self.key_id.clone(),
            owner_id: self.owner_id.clone(),
            issuer: self.issuer.clone(),
            tags: self.tags.clone(),
            pinned: self.pinned,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a strict release adjustment in status open.
pub fn new_strict_release_adjustment(code: &str, name: &str) -> StrictReleaseAdjustment {
    StrictReleaseAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        visible: false,
        grace_days: 0,
        key_id: "".to_string(),
        owner_id: "".to_string(),
        issuer: "".to_string(),
        tags: Vec::new(),
        pinned: false,
        updated_at: 0,
        status: StrictReleaseAdjustmentStatus::Open,
    }
}
