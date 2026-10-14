use super::*;

use acme_kit::text;

/// Where a grant request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantRequestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl GrantRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for GrantRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant request as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantRequest {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub score: f64,
    pub issuer: String,
    pub name: String,
    pub ends_at: i64,
    pub ttl_seconds: i64,
    pub priority: i64,
    pub country: String,
    pub scope: String,
    pub status: GrantRequestStatus,
}

impl GrantRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantRequestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantRequestStatus) -> bool {
        match self.status {
            GrantRequestStatus::Proposed => {
                match next {
                    GrantRequestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantRequestStatus::Approved => {
                match next {
                    GrantRequestStatus::Rejected | GrantRequestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantRequestStatus::Rejected => {
                match next {
                    GrantRequestStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: GrantRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "grant request", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.ends_at as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantRequest {
        GrantRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            score: self.score,
            issuer: self.issuer.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            ttl_seconds: self.ttl_seconds,
            priority: self.priority,
            country: self.country.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a grant request in status proposed.
pub fn new_grant_request(code: &str, label: &str) -> GrantRequest {
    GrantRequest {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        score: 0.0,
        issuer: "".to_string(),
        name: "".to_string(),
        ends_at: 0,
        ttl_seconds: 0,
        priority: 0,
        country: "".to_string(),
        scope: "".to_string(),
        status: GrantRequestStatus::Proposed,
    }
}
