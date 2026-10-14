use super::*;

use acme_kit::text;

/// Where a approval preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalPreferenceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ApprovalPreferenceStatus {
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

impl std::fmt::Display for ApprovalPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One approval preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalPreference {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub tags: Vec<String>,
    pub burst: i64,
    pub approved: bool,
    pub quantity: i64,
    pub issuer: String,
    pub pinned: bool,
    pub name: String,
    pub batch_size: i64,
    pub status: ApprovalPreferenceStatus,
}

impl ApprovalPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalPreferenceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalPreferenceStatus) -> bool {
        match self.status {
            ApprovalPreferenceStatus::Open => {
                match next {
                    ApprovalPreferenceStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalPreferenceStatus::InReview => {
                return next == ApprovalPreferenceStatus::Resolved || next == ApprovalPreferenceStatus::Archived;
            }
            ApprovalPreferenceStatus::Resolved => {
                return next == ApprovalPreferenceStatus::Reopened;
            }
            ApprovalPreferenceStatus::Reopened => {
                match next {
                    ApprovalPreferenceStatus::Archived => {
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

    pub fn move_to(&mut self, next: ApprovalPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.score + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalPreference {
        ApprovalPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            tags: self.tags.clone(),
            burst: self.burst,
            approved: self.approved,
            quantity: self.quantity,
            issuer: self.issuer.clone(),
            pinned: self.pinned,
            name: self.name.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a approval preference in status open.
pub fn new_approval_preference(code: &str, label: &str) -> ApprovalPreference {
    ApprovalPreference {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        tags: Vec::new(),
        burst: 0,
        approved: false,
        quantity: 0,
        issuer: "".to_string(),
        pinned: false,
        name: "".to_string(),
        batch_size: 0,
        status: ApprovalPreferenceStatus::Open,
    }
}
