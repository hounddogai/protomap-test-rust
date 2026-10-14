use super::*;

use acme_kit::text;

/// Where a fallback egress agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackEgressAgreementStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FallbackEgressAgreementStatus {
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

impl std::fmt::Display for FallbackEgressAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback egress agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressAgreement {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub limit: i64,
    pub minimum: f64,
    pub key_id: String,
    pub scope: String,
    pub status: FallbackEgressAgreementStatus,
}

impl FallbackEgressAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackEgressAgreementStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackEgressAgreementStatus) -> bool {
        match self.status {
            FallbackEgressAgreementStatus::Idle => {
                match next {
                    FallbackEgressAgreementStatus::Warming | FallbackEgressAgreementStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackEgressAgreementStatus::Warming => {
                return next == FallbackEgressAgreementStatus::Draining;
            }
            FallbackEgressAgreementStatus::Ready => {
                return next == FallbackEgressAgreementStatus::Draining
                    || next == FallbackEgressAgreementStatus::Stopped;
            }
            FallbackEgressAgreementStatus::Draining => {
                return next == FallbackEgressAgreementStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackEgressAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback egress agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.minimum - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackEgressAgreement {
        FallbackEgressAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            limit: self.limit,
            minimum: self.minimum,
            key_id: self.key_id.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a fallback egress agreement in status idle.
pub fn new_fallback_egress_agreement(code: &str, name: &str) -> FallbackEgressAgreement {
    FallbackEgressAgreement {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        limit: 0,
        minimum: 0.0,
        key_id: "".to_string(),
        scope: "".to_string(),
        status: FallbackEgressAgreementStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_fallback_egress_agreement("code-271", "name-484");
        assert_eq!(item.status, FallbackEgressAgreementStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_fallback_egress_agreement("code-271", "name-484");
        if item.move_to(FallbackEgressAgreementStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackEgressAgreementStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_fallback_egress_agreement("code-271", "name-484");
        item.status = FallbackEgressAgreementStatus::Stopped;
        assert!(
            item.move_to(FallbackEgressAgreementStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
