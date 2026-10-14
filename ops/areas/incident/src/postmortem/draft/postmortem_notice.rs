use super::*;

use acme_kit::text;

/// Where a postmortem notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemNoticeStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PostmortemNoticeStatus {
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

impl std::fmt::Display for PostmortemNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a postmortem notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemNotice {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub key_id: String,
    pub external_ref: String,
    pub revision: i64,
    pub ratio: f64,
    pub rate: f64,
    pub status: PostmortemNoticeStatus,
}

impl PostmortemNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemNoticeStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemNoticeStatus) -> bool {
        match self.status {
            PostmortemNoticeStatus::Open => {
                return next == PostmortemNoticeStatus::Resolved;
            }
            PostmortemNoticeStatus::InReview => {
                return next == PostmortemNoticeStatus::Reopened;
            }
            PostmortemNoticeStatus::Resolved => {
                return next == PostmortemNoticeStatus::Archived;
            }
            PostmortemNoticeStatus::Reopened => {
                return next == PostmortemNoticeStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PostmortemNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        (self.ratio * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemNotice {
        PostmortemNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            key_id: self.key_id.clone(),
            external_ref: self.external_ref.clone(),
            revision: self.revision,
            ratio: self.ratio,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a postmortem notice in status open.
pub fn new_postmortem_notice(code: &str, title: &str) -> PostmortemNotice {
    PostmortemNotice {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        key_id: "".to_string(),
        external_ref: "".to_string(),
        revision: 0,
        ratio: 0.0,
        rate: 0.0,
        status: PostmortemNoticeStatus::Open,
    }
}
