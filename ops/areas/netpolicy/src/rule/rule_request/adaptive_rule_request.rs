use super::*;

use acme_kit::text;

/// Where a adaptive rule request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRuleRequestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AdaptiveRuleRequestStatus {
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

impl std::fmt::Display for AdaptiveRuleRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive rule request and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleRequest {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub issuer: String,
    pub amount: f64,
    pub ttl_seconds: i64,
    pub sequence: i64,
    pub limit: i64,
    pub status: AdaptiveRuleRequestStatus,
}

impl AdaptiveRuleRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRuleRequestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRuleRequestStatus) -> bool {
        match self.status {
            AdaptiveRuleRequestStatus::Idle => {
                match next {
                    AdaptiveRuleRequestStatus::Warming | AdaptiveRuleRequestStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRuleRequestStatus::Warming => {
                match next {
                    AdaptiveRuleRequestStatus::Draining | AdaptiveRuleRequestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRuleRequestStatus::Ready => {
                return next == AdaptiveRuleRequestStatus::Draining || next == AdaptiveRuleRequestStatus::Stopped;
            }
            AdaptiveRuleRequestStatus::Draining => {
                match next {
                    AdaptiveRuleRequestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: AdaptiveRuleRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive rule request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive rule request", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.amount + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRuleRequest {
        AdaptiveRuleRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            issuer: self.issuer.clone(),
            amount: self.amount,
            ttl_seconds: self.ttl_seconds,
            sequence: self.sequence,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a adaptive rule request in status idle.
pub fn new_adaptive_rule_request(code: &str, name: &str) -> AdaptiveRuleRequest {
    AdaptiveRuleRequest {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        issuer: "".to_string(),
        amount: 0.0,
        ttl_seconds: 0,
        sequence: 0,
        limit: 0,
        status: AdaptiveRuleRequestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_adaptive_rule_request("code-680", "name-765");
        assert_eq!(item.status, AdaptiveRuleRequestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_adaptive_rule_request("code-680", "name-765");
        if item.move_to(AdaptiveRuleRequestStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveRuleRequestStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_adaptive_rule_request("code-680", "name-765");
        item.status = AdaptiveRuleRequestStatus::Stopped;
        assert!(
            item.move_to(AdaptiveRuleRequestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
