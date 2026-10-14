use super::*;

use acme_kit::text;

/// Where a usage digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageDigestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl UsageDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for UsageDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a usage digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageDigest {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub approved: bool,
    pub starts_at: i64,
    pub rotation_days: i64,
    pub tenant_id: String,
    pub status: UsageDigestStatus,
}

impl UsageDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageDigestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageDigestStatus) -> bool {
        match self.status {
            UsageDigestStatus::Planned => {
                match next {
                    UsageDigestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageDigestStatus::Scheduled => {
                match next {
                    UsageDigestStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageDigestStatus::InProgress => {
                return next == UsageDigestStatus::Completed || next == UsageDigestStatus::Cancelled;
            }
            UsageDigestStatus::Completed => {
                return next == UsageDigestStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.starts_at as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageDigest {
        UsageDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            approved: self.approved,
            starts_at: self.starts_at,
            rotation_days: self.rotation_days,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a usage digest in status planned.
pub fn new_usage_digest(code: &str, label: &str) -> UsageDigest {
    UsageDigest {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        approved: false,
        starts_at: 0,
        rotation_days: 0,
        tenant_id: "".to_string(),
        status: UsageDigestStatus::Planned,
    }
}
