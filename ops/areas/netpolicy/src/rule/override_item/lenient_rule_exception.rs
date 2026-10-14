use super::*;

use acme_kit::text;

/// Where a lenient rule exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientRuleExceptionStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientRuleExceptionStatus {
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

impl std::fmt::Display for LenientRuleExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient rule exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleException {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub quota_units: i64,
    pub ratio: f64,
    pub status: LenientRuleExceptionStatus,
}

impl LenientRuleException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientRuleExceptionStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientRuleExceptionStatus) -> bool {
        match self.status {
            LenientRuleExceptionStatus::Idle => {
                match next {
                    LenientRuleExceptionStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRuleExceptionStatus::Warming => {
                return next == LenientRuleExceptionStatus::Stopped;
            }
            LenientRuleExceptionStatus::Ready => {
                return next == LenientRuleExceptionStatus::Draining || next == LenientRuleExceptionStatus::Stopped;
            }
            LenientRuleExceptionStatus::Draining => {
                return next == LenientRuleExceptionStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientRuleExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient rule exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "lenient rule exception", self.code)
    }

    pub fn net_value(&self) -> f64 {
        (self.ratio * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientRuleException {
        LenientRuleException {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            quota_units: self.quota_units,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient rule exception in status idle.
pub fn new_lenient_rule_exception(code: &str, label: &str) -> LenientRuleException {
    LenientRuleException {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        quota_units: 0,
        ratio: 0.0,
        status: LenientRuleExceptionStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_lenient_rule_exception("code-846", "label-732");
        assert_eq!(item.status, LenientRuleExceptionStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_lenient_rule_exception("code-846", "label-732");
        if item.move_to(LenientRuleExceptionStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientRuleExceptionStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_lenient_rule_exception("code-846", "label-732");
        item.status = LenientRuleExceptionStatus::Stopped;
        assert!(
            item.move_to(LenientRuleExceptionStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
