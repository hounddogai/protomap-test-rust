use super::*;

use acme_kit::text;

/// Where a adaptive identity baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveIdentityBaselineStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveIdentityBaselineStatus {
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

impl std::fmt::Display for AdaptiveIdentityBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive identity baseline and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentityBaseline {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub pinned: bool,
    pub batch_size: i64,
    pub weight: f64,
    pub ttl_seconds: i64,
    pub key_id: String,
    pub grace_days: i64,
    pub quantity: i64,
    pub status: AdaptiveIdentityBaselineStatus,
}

impl AdaptiveIdentityBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveIdentityBaselineStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveIdentityBaselineStatus) -> bool {
        match self.status {
            AdaptiveIdentityBaselineStatus::Open => {
                return next == AdaptiveIdentityBaselineStatus::Reopened;
            }
            AdaptiveIdentityBaselineStatus::InReview => {
                match next {
                    AdaptiveIdentityBaselineStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveIdentityBaselineStatus::Resolved => {
                match next {
                    AdaptiveIdentityBaselineStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveIdentityBaselineStatus::Reopened => {
                return next == AdaptiveIdentityBaselineStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveIdentityBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive identity baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive identity baseline", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.grace_days as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveIdentityBaseline {
        AdaptiveIdentityBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            pinned: self.pinned,
            batch_size: self.batch_size,
            weight: self.weight,
            ttl_seconds: self.ttl_seconds,
            key_id: self.key_id.clone(),
            grace_days: self.grace_days,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a adaptive identity baseline in status open.
pub fn new_adaptive_identity_baseline(code: &str, name: &str) -> AdaptiveIdentityBaseline {
    AdaptiveIdentityBaseline {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        pinned: false,
        batch_size: 0,
        weight: 0.0,
        ttl_seconds: 0,
        key_id: "".to_string(),
        grace_days: 0,
        quantity: 0,
        status: AdaptiveIdentityBaselineStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_adaptive_identity_baseline("code-99", "name-578");
        assert_eq!(item.status, AdaptiveIdentityBaselineStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_adaptive_identity_baseline("code-99", "name-578");
        if item.move_to(AdaptiveIdentityBaselineStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveIdentityBaselineStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_adaptive_identity_baseline("code-99", "name-578");
        item.status = AdaptiveIdentityBaselineStatus::Archived;
        assert!(
            item.move_to(AdaptiveIdentityBaselineStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
