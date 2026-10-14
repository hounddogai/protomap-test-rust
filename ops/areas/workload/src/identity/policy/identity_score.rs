use super::*;

use acme_kit::text;

/// Where a identity score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityScoreStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl IdentityScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for IdentityScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityScore {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub host: String,
    pub cluster: String,
    pub ttl_seconds: i64,
    pub locale: String,
    pub grace_days: i64,
    pub status: IdentityScoreStatus,
}

impl IdentityScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityScoreStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityScoreStatus) -> bool {
        match self.status {
            IdentityScoreStatus::Requested => {
                return next == IdentityScoreStatus::Granted || next == IdentityScoreStatus::Revoked;
            }
            IdentityScoreStatus::Granted => {
                match next {
                    IdentityScoreStatus::Revoked | IdentityScoreStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityScoreStatus::Revoked => {
                match next {
                    IdentityScoreStatus::Expired => {
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

    pub fn move_to(&mut self, next: IdentityScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityScore {
        IdentityScore {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            host: self.host.clone(),
            cluster: self.cluster.clone(),
            ttl_seconds: self.ttl_seconds,
            locale: self.locale.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a identity score in status requested.
pub fn new_identity_score(code: &str, label: &str) -> IdentityScore {
    IdentityScore {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        host: "".to_string(),
        cluster: "".to_string(),
        ttl_seconds: 0,
        locale: "".to_string(),
        grace_days: 0,
        status: IdentityScoreStatus::Requested,
    }
}
