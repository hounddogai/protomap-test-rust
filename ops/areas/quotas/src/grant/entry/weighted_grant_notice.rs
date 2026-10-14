use super::*;

use acme_kit::text;

/// Where a weighted grant notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedGrantNoticeStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl WeightedGrantNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for WeightedGrantNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted grant notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedGrantNotice {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub minimum: f64,
    pub retries: i64,
    pub pinned: bool,
    pub capacity: i64,
    pub status: WeightedGrantNoticeStatus,
}

impl WeightedGrantNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedGrantNoticeStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedGrantNoticeStatus) -> bool {
        match self.status {
            WeightedGrantNoticeStatus::Requested => {
                return next == WeightedGrantNoticeStatus::Expired;
            }
            WeightedGrantNoticeStatus::Granted => {
                return next == WeightedGrantNoticeStatus::Revoked || next == WeightedGrantNoticeStatus::Expired;
            }
            WeightedGrantNoticeStatus::Revoked => {
                return next == WeightedGrantNoticeStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedGrantNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted grant notice {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.retries as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedGrantNotice {
        WeightedGrantNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            minimum: self.minimum,
            retries: self.retries,
            pinned: self.pinned,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted grant notice in status requested.
pub fn new_weighted_grant_notice(code: &str, label: &str) -> WeightedGrantNotice {
    WeightedGrantNotice {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        minimum: 0.0,
        retries: 0,
        pinned: false,
        capacity: 0,
        status: WeightedGrantNoticeStatus::Requested,
    }
}
