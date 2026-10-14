use super::*;

use acme_kit::text;

/// Where a regional redaction agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalRedactionAgreementStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalRedactionAgreementStatus {
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

impl std::fmt::Display for RegionalRedactionAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional redaction agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRedactionAgreement {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub sequence: i64,
    pub visible: bool,
    pub budget: f64,
    pub version: i64,
    pub status: RegionalRedactionAgreementStatus,
}

impl RegionalRedactionAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalRedactionAgreementStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalRedactionAgreementStatus) -> bool {
        match self.status {
            RegionalRedactionAgreementStatus::Idle => {
                match next {
                    RegionalRedactionAgreementStatus::Draining | RegionalRedactionAgreementStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRedactionAgreementStatus::Warming => {
                match next {
                    RegionalRedactionAgreementStatus::Ready | RegionalRedactionAgreementStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRedactionAgreementStatus::Ready => {
                match next {
                    RegionalRedactionAgreementStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRedactionAgreementStatus::Draining => {
                match next {
                    RegionalRedactionAgreementStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RegionalRedactionAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional redaction agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional redaction agreement", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.version as f64) * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalRedactionAgreement {
        RegionalRedactionAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            sequence: self.sequence,
            visible: self.visible,
            budget: self.budget,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional redaction agreement in status idle.
pub fn new_regional_redaction_agreement(code: &str, name: &str) -> RegionalRedactionAgreement {
    RegionalRedactionAgreement {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        sequence: 0,
        visible: false,
        budget: 0.0,
        version: 0,
        status: RegionalRedactionAgreementStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_redaction_agreement("code-916", "name-602");
        assert_eq!(item.status, RegionalRedactionAgreementStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_regional_redaction_agreement("code-916", "name-602");
        if item.move_to(RegionalRedactionAgreementStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalRedactionAgreementStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_redaction_agreement("code-916", "name-602");
        item.status = RegionalRedactionAgreementStatus::Stopped;
        assert!(
            item.move_to(RegionalRedactionAgreementStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
