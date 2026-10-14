use super::*;

use acme_kit::text;

/// Where a batched release template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedReleaseTemplateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedReleaseTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedReleaseTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched release template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReleaseTemplate {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub maximum: f64,
    pub locale: String,
    pub issuer: String,
    pub capacity: i64,
    pub burst: i64,
    pub status: BatchedReleaseTemplateStatus,
}

impl BatchedReleaseTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedReleaseTemplateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedReleaseTemplateStatus) -> bool {
        match self.status {
            BatchedReleaseTemplateStatus::Requested => {
                return next == BatchedReleaseTemplateStatus::Expired;
            }
            BatchedReleaseTemplateStatus::Granted => {
                match next {
                    BatchedReleaseTemplateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedReleaseTemplateStatus::Revoked => {
                return next == BatchedReleaseTemplateStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedReleaseTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched release template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched release template", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.burst as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedReleaseTemplate {
        BatchedReleaseTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            maximum: self.maximum,
            locale: self.locale.clone(),
            issuer: self.issuer.clone(),
            capacity: self.capacity,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a batched release template in status requested.
pub fn new_batched_release_template(code: &str, title: &str) -> BatchedReleaseTemplate {
    BatchedReleaseTemplate {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        maximum: 0.0,
        locale: "".to_string(),
        issuer: "".to_string(),
        capacity: 0,
        burst: 0,
        status: BatchedReleaseTemplateStatus::Requested,
    }
}
