use super::*;

use acme_kit::text;

/// Where a redaction notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionNoticeStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RedactionNoticeStatus {
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

impl std::fmt::Display for RedactionNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a redaction notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionNotice {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub key_id: String,
    pub source: String,
    pub country: String,
    pub title: String,
    pub label: String,
    pub revision: i64,
    pub margin: f64,
    pub visible: bool,
    pub status: RedactionNoticeStatus,
}

impl RedactionNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionNoticeStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionNoticeStatus) -> bool {
        match self.status {
            RedactionNoticeStatus::Open => {
                match next {
                    RedactionNoticeStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionNoticeStatus::InReview => {
                return next == RedactionNoticeStatus::Resolved;
            }
            RedactionNoticeStatus::Resolved => {
                match next {
                    RedactionNoticeStatus::Reopened | RedactionNoticeStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionNoticeStatus::Reopened => {
                return next == RedactionNoticeStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RedactionNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "redaction notice", self.code)
    }

    pub fn net_load(&self) -> f64 {
        (self.margin * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionNotice {
        RedactionNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            key_id: self.key_id.clone(),
            source: self.source.clone(),
            country: self.country.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            revision: self.revision,
            margin: self.margin,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a redaction notice in status open.
pub fn new_redaction_notice(code: &str, name: &str) -> RedactionNotice {
    RedactionNotice {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        key_id: "".to_string(),
        source: "".to_string(),
        country: "".to_string(),
        title: "".to_string(),
        label: "".to_string(),
        revision: 0,
        margin: 0.0,
        visible: false,
        status: RedactionNoticeStatus::Open,
    }
}
