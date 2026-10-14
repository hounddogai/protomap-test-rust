use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a fallback identity quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackIdentityQuotaStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackIdentityQuotaStatus {
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

impl std::fmt::Display for FallbackIdentityQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback identity quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityQuota {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub attributes: BTreeMap<String, String>,
    pub score: f64,
    pub discount: f64,
    pub ends_at: i64,
    pub region: String,
    pub ttl_seconds: i64,
    pub budget: f64,
    pub attempts: i64,
    pub status: FallbackIdentityQuotaStatus,
}

impl FallbackIdentityQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackIdentityQuotaStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackIdentityQuotaStatus) -> bool {
        match self.status {
            FallbackIdentityQuotaStatus::Open => {
                return next == FallbackIdentityQuotaStatus::Archived;
            }
            FallbackIdentityQuotaStatus::InReview => {
                return next == FallbackIdentityQuotaStatus::Resolved;
            }
            FallbackIdentityQuotaStatus::Resolved => {
                match next {
                    FallbackIdentityQuotaStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackIdentityQuotaStatus::Reopened => {
                return next == FallbackIdentityQuotaStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackIdentityQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback identity quota {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "fallback identity quota", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.attempts as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackIdentityQuota {
        FallbackIdentityQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            attributes: self.attributes.clone(),
            score: self.score,
            discount: self.discount,
            ends_at: self.ends_at,
            region: self.region.clone(),
            ttl_seconds: self.ttl_seconds,
            budget: self.budget,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a fallback identity quota in status open.
pub fn new_fallback_identity_quota(code: &str, name: &str) -> FallbackIdentityQuota {
    FallbackIdentityQuota {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        attributes: BTreeMap::new(),
        score: 0.0,
        discount: 0.0,
        ends_at: 0,
        region: "".to_string(),
        ttl_seconds: 0,
        budget: 0.0,
        attempts: 0,
        status: FallbackIdentityQuotaStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_fallback_identity_quota("code-417", "name-700");
        assert_eq!(item.status, FallbackIdentityQuotaStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_fallback_identity_quota("code-417", "name-700");
        if item.move_to(FallbackIdentityQuotaStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackIdentityQuotaStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_fallback_identity_quota("code-417", "name-700");
        item.status = FallbackIdentityQuotaStatus::Archived;
        assert!(
            item.move_to(FallbackIdentityQuotaStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
