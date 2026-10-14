use super::*;

use acme_kit::text;

/// Where a legacy canary agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyCanaryAgreementStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LegacyCanaryAgreementStatus {
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

impl std::fmt::Display for LegacyCanaryAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy canary agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryAgreement {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub attempts: i64,
    pub unit: String,
    pub budget: f64,
    pub status: LegacyCanaryAgreementStatus,
}

impl LegacyCanaryAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyCanaryAgreementStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyCanaryAgreementStatus) -> bool {
        match self.status {
            LegacyCanaryAgreementStatus::Idle => {
                match next {
                    LegacyCanaryAgreementStatus::Draining | LegacyCanaryAgreementStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyCanaryAgreementStatus::Warming => {
                match next {
                    LegacyCanaryAgreementStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyCanaryAgreementStatus::Ready => {
                return next == LegacyCanaryAgreementStatus::Draining || next == LegacyCanaryAgreementStatus::Stopped;
            }
            LegacyCanaryAgreementStatus::Draining => {
                match next {
                    LegacyCanaryAgreementStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LegacyCanaryAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy canary agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.ratio + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyCanaryAgreement {
        LegacyCanaryAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            attempts: self.attempts,
            unit: self.unit.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a legacy canary agreement in status idle.
pub fn new_legacy_canary_agreement(code: &str, title: &str) -> LegacyCanaryAgreement {
    LegacyCanaryAgreement {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        attempts: 0,
        unit: "".to_string(),
        budget: 0.0,
        status: LegacyCanaryAgreementStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_legacy_canary_agreement("code-540", "title-281");
        assert_eq!(item.status, LegacyCanaryAgreementStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_legacy_canary_agreement("code-540", "title-281");
        if item.move_to(LegacyCanaryAgreementStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyCanaryAgreementStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_legacy_canary_agreement("code-540", "title-281");
        item.status = LegacyCanaryAgreementStatus::Stopped;
        assert!(
            item.move_to(LegacyCanaryAgreementStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
