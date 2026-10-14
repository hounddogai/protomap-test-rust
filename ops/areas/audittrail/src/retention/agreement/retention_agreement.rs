use super::*;

use acme_kit::text;

/// Where a retention agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionAgreementStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RetentionAgreementStatus {
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

impl std::fmt::Display for RetentionAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a retention agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAgreement {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub margin: f64,
    pub principal: String,
    pub version: i64,
    pub status: RetentionAgreementStatus,
}

impl RetentionAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionAgreementStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionAgreementStatus) -> bool {
        match self.status {
            RetentionAgreementStatus::Idle => {
                match next {
                    RetentionAgreementStatus::Warming | RetentionAgreementStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionAgreementStatus::Warming => {
                match next {
                    RetentionAgreementStatus::Ready | RetentionAgreementStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionAgreementStatus::Ready => {
                match next {
                    RetentionAgreementStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionAgreementStatus::Draining => {
                return next == RetentionAgreementStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.version as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionAgreement {
        RetentionAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            margin: self.margin,
            principal: self.principal.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a retention agreement in status idle.
pub fn new_retention_agreement(code: &str, label: &str) -> RetentionAgreement {
    RetentionAgreement {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        margin: 0.0,
        principal: "".to_string(),
        version: 0,
        status: RetentionAgreementStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_retention_agreement("code-754", "label-508");
        assert_eq!(item.status, RetentionAgreementStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_retention_agreement("code-754", "label-508");
        if item.move_to(RetentionAgreementStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionAgreementStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_retention_agreement("code-754", "label-508");
        item.status = RetentionAgreementStatus::Stopped;
        assert!(
            item.move_to(RetentionAgreementStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
