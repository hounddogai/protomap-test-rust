use super::*;

use acme_kit::text;

/// Where a fallback suspension agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackSuspensionAgreementStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FallbackSuspensionAgreementStatus {
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

impl std::fmt::Display for FallbackSuspensionAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback suspension agreement as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSuspensionAgreement {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub ttl_seconds: i64,
    pub priority: i64,
    pub status: FallbackSuspensionAgreementStatus,
}

impl FallbackSuspensionAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackSuspensionAgreementStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackSuspensionAgreementStatus) -> bool {
        match self.status {
            FallbackSuspensionAgreementStatus::Idle => {
                match next {
                    FallbackSuspensionAgreementStatus::Ready | FallbackSuspensionAgreementStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackSuspensionAgreementStatus::Warming => {
                match next {
                    FallbackSuspensionAgreementStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackSuspensionAgreementStatus::Ready => {
                match next {
                    FallbackSuspensionAgreementStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackSuspensionAgreementStatus::Draining => {
                return next == FallbackSuspensionAgreementStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackSuspensionAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback suspension agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.quota_units as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackSuspensionAgreement {
        FallbackSuspensionAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            ttl_seconds: self.ttl_seconds,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback suspension agreement in status idle.
pub fn new_fallback_suspension_agreement(code: &str, name: &str) -> FallbackSuspensionAgreement {
    FallbackSuspensionAgreement {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        ttl_seconds: 0,
        priority: 0,
        status: FallbackSuspensionAgreementStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_fallback_suspension_agreement("code-424", "name-484");
        assert_eq!(item.status, FallbackSuspensionAgreementStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_fallback_suspension_agreement("code-424", "name-484");
        if item.move_to(FallbackSuspensionAgreementStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackSuspensionAgreementStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_fallback_suspension_agreement("code-424", "name-484");
        item.status = FallbackSuspensionAgreementStatus::Stopped;
        assert!(
            item.move_to(FallbackSuspensionAgreementStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
