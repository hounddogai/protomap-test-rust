use super::*;

use acme_kit::text;

/// Where a postmortem draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemDraftStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PostmortemDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PostmortemDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem draft as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemDraft {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub version: i64,
    pub retries: i64,
    pub archived: bool,
    pub port: i64,
    pub quota_units: i64,
    pub title: String,
    pub status: PostmortemDraftStatus,
}

impl PostmortemDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemDraftStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemDraftStatus) -> bool {
        match self.status {
            PostmortemDraftStatus::Requested => {
                match next {
                    PostmortemDraftStatus::Revoked | PostmortemDraftStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemDraftStatus::Granted => {
                return next == PostmortemDraftStatus::Revoked;
            }
            PostmortemDraftStatus::Revoked => {
                match next {
                    PostmortemDraftStatus::Expired => {
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

    pub fn move_to(&mut self, next: PostmortemDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "postmortem draft", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.capacity as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemDraft {
        PostmortemDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            version: self.version,
            retries: self.retries,
            archived: self.archived,
            port: self.port,
            quota_units: self.quota_units,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a postmortem draft in status requested.
pub fn new_postmortem_draft(code: &str, label: &str) -> PostmortemDraft {
    PostmortemDraft {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        version: 0,
        retries: 0,
        archived: false,
        port: 0,
        quota_units: 0,
        title: "".to_string(),
        status: PostmortemDraftStatus::Requested,
    }
}
