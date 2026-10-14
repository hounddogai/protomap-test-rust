use super::*;

use acme_kit::text;

/// Where a postmortem request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemRequestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PostmortemRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PostmortemRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem request as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemRequest {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub quota_units: i64,
    pub duration_minutes: i64,
    pub cluster: String,
    pub status: PostmortemRequestStatus,
}

impl PostmortemRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemRequestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemRequestStatus) -> bool {
        match self.status {
            PostmortemRequestStatus::Created => {
                return next == PostmortemRequestStatus::Validated || next == PostmortemRequestStatus::Published;
            }
            PostmortemRequestStatus::Validated => {
                match next {
                    PostmortemRequestStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemRequestStatus::Published => {
                match next {
                    PostmortemRequestStatus::Retired => {
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

    pub fn move_to(&mut self, next: PostmortemRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "postmortem request", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemRequest {
        PostmortemRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            quota_units: self.quota_units,
            duration_minutes: self.duration_minutes,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a postmortem request in status created.
pub fn new_postmortem_request(code: &str, label: &str) -> PostmortemRequest {
    PostmortemRequest {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        quota_units: 0,
        duration_minutes: 0,
        cluster: "".to_string(),
        status: PostmortemRequestStatus::Created,
    }
}
