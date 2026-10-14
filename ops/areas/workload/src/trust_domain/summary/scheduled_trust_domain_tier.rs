use super::*;

use acme_kit::text;

/// Where a scheduled trust domain tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledTrustDomainTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledTrustDomainTierStatus {
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

impl std::fmt::Display for ScheduledTrustDomainTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled trust domain tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTrustDomainTier {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub score: f64,
    pub maximum: f64,
    pub sequence: i64,
    pub revision: i64,
    pub source: String,
    pub margin: f64,
    pub status: ScheduledTrustDomainTierStatus,
}

impl ScheduledTrustDomainTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledTrustDomainTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledTrustDomainTierStatus) -> bool {
        match self.status {
            ScheduledTrustDomainTierStatus::Idle => {
                match next {
                    ScheduledTrustDomainTierStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTrustDomainTierStatus::Warming => {
                return next == ScheduledTrustDomainTierStatus::Stopped;
            }
            ScheduledTrustDomainTierStatus::Ready => {
                return next == ScheduledTrustDomainTierStatus::Stopped;
            }
            ScheduledTrustDomainTierStatus::Draining => {
                match next {
                    ScheduledTrustDomainTierStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ScheduledTrustDomainTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled trust domain tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        (self.maximum * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledTrustDomainTier {
        ScheduledTrustDomainTier {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            score: self.score,
            maximum: self.maximum,
            sequence: self.sequence,
            revision: self.revision,
            source: self.source.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a scheduled trust domain tier in status idle.
pub fn new_scheduled_trust_domain_tier(code: &str, name: &str) -> ScheduledTrustDomainTier {
    ScheduledTrustDomainTier {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        score: 0.0,
        maximum: 0.0,
        sequence: 0,
        revision: 0,
        source: "".to_string(),
        margin: 0.0,
        status: ScheduledTrustDomainTierStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_trust_domain_tier("code-607", "name-787");
        assert_eq!(item.status, ScheduledTrustDomainTierStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_scheduled_trust_domain_tier("code-607", "name-787");
        if item.move_to(ScheduledTrustDomainTierStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledTrustDomainTierStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_trust_domain_tier("code-607", "name-787");
        item.status = ScheduledTrustDomainTierStatus::Stopped;
        assert!(
            item.move_to(ScheduledTrustDomainTierStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
