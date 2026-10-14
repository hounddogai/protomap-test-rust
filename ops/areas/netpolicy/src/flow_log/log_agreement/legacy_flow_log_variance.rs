use super::*;

use acme_kit::text;

/// Where a legacy flow log variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyFlowLogVarianceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LegacyFlowLogVarianceStatus {
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

impl std::fmt::Display for LegacyFlowLogVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy flow log variance. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogVariance {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub unit: String,
    pub cipher: String,
    pub contact_email: String,
    pub segment: String,
    pub status: LegacyFlowLogVarianceStatus,
}

impl LegacyFlowLogVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyFlowLogVarianceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyFlowLogVarianceStatus) -> bool {
        match self.status {
            LegacyFlowLogVarianceStatus::Idle => {
                return next == LegacyFlowLogVarianceStatus::Warming || next == LegacyFlowLogVarianceStatus::Stopped;
            }
            LegacyFlowLogVarianceStatus::Warming => {
                return next == LegacyFlowLogVarianceStatus::Stopped;
            }
            LegacyFlowLogVarianceStatus::Ready => {
                return next == LegacyFlowLogVarianceStatus::Stopped;
            }
            LegacyFlowLogVarianceStatus::Draining => {
                return next == LegacyFlowLogVarianceStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyFlowLogVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy flow log variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> LegacyFlowLogVariance {
        LegacyFlowLogVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            unit: self.unit.clone(),
            cipher: self.cipher.clone(),
            contact_email: self.contact_email.clone(),
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a legacy flow log variance in status idle.
pub fn new_legacy_flow_log_variance(code: &str, title: &str) -> LegacyFlowLogVariance {
    LegacyFlowLogVariance {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        unit: "".to_string(),
        cipher: "".to_string(),
        contact_email: "".to_string(),
        segment: "".to_string(),
        status: LegacyFlowLogVarianceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_legacy_flow_log_variance("code-926", "title-339");
        assert_eq!(item.status, LegacyFlowLogVarianceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_legacy_flow_log_variance("code-926", "title-339");
        if item.move_to(LegacyFlowLogVarianceStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyFlowLogVarianceStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_legacy_flow_log_variance("code-926", "title-339");
        item.status = LegacyFlowLogVarianceStatus::Stopped;
        assert!(
            item.move_to(LegacyFlowLogVarianceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
