use super::*;

use acme_kit::text;

/// Where a capped ttl policy tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedTtlPolicyTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CappedTtlPolicyTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for CappedTtlPolicyTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped ttl policy tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTtlPolicyTier {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub description: String,
    pub retries: i64,
    pub port: i64,
    pub category: String,
    pub threshold: f64,
    pub status: CappedTtlPolicyTierStatus,
}

impl CappedTtlPolicyTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedTtlPolicyTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedTtlPolicyTierStatus) -> bool {
        match self.status {
            CappedTtlPolicyTierStatus::Idle => {
                match next {
                    CappedTtlPolicyTierStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedTtlPolicyTierStatus::Warming => {
                return next == CappedTtlPolicyTierStatus::Draining;
            }
            CappedTtlPolicyTierStatus::Ready => {
                return next == CappedTtlPolicyTierStatus::Draining || next == CappedTtlPolicyTierStatus::Stopped;
            }
            CappedTtlPolicyTierStatus::Draining => {
                match next {
                    CappedTtlPolicyTierStatus::Stopped => {
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

    pub fn move_to(&mut self, next: CappedTtlPolicyTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped ttl policy tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.port as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedTtlPolicyTier {
        CappedTtlPolicyTier {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            description: self.description.clone(),
            retries: self.retries,
            port: self.port,
            category: self.category.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a capped ttl policy tier in status idle.
pub fn new_capped_ttl_policy_tier(code: &str, name: &str) -> CappedTtlPolicyTier {
    CappedTtlPolicyTier {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        description: "".to_string(),
        retries: 0,
        port: 0,
        category: "".to_string(),
        threshold: 0.0,
        status: CappedTtlPolicyTierStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_capped_ttl_policy_tier("code-730", "name-399");
        assert_eq!(item.status, CappedTtlPolicyTierStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_capped_ttl_policy_tier("code-730", "name-399");
        if item.move_to(CappedTtlPolicyTierStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedTtlPolicyTierStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_capped_ttl_policy_tier("code-730", "name-399");
        item.status = CappedTtlPolicyTierStatus::Stopped;
        assert!(
            item.move_to(CappedTtlPolicyTierStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
