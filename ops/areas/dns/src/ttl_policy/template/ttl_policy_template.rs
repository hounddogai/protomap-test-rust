use super::*;

use acme_kit::text;

/// Where a ttl policy template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyTemplateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TtlPolicyTemplateStatus {
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

impl std::fmt::Display for TtlPolicyTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a ttl policy template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyTemplate {
    pub code: String,
    pub label: String,
    pub quota_units: i64,
    pub score: f64,
    pub ttl_seconds: i64,
    pub cost: f64,
    pub status: TtlPolicyTemplateStatus,
}

impl TtlPolicyTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyTemplateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyTemplateStatus) -> bool {
        match self.status {
            TtlPolicyTemplateStatus::Idle => {
                return next == TtlPolicyTemplateStatus::Ready || next == TtlPolicyTemplateStatus::Stopped;
            }
            TtlPolicyTemplateStatus::Warming => {
                return next == TtlPolicyTemplateStatus::Draining || next == TtlPolicyTemplateStatus::Stopped;
            }
            TtlPolicyTemplateStatus::Ready => {
                return next == TtlPolicyTemplateStatus::Draining || next == TtlPolicyTemplateStatus::Stopped;
            }
            TtlPolicyTemplateStatus::Draining => {
                return next == TtlPolicyTemplateStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "ttl policy template", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.cost - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyTemplate {
        TtlPolicyTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            score: self.score,
            ttl_seconds: self.ttl_seconds,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a ttl policy template in status idle.
pub fn new_ttl_policy_template(code: &str, label: &str) -> TtlPolicyTemplate {
    TtlPolicyTemplate {
        code: code.to_string(),
        label: label.to_string(),
        quota_units: 0,
        score: 0.0,
        ttl_seconds: 0,
        cost: 0.0,
        status: TtlPolicyTemplateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_ttl_policy_template("code-491", "label-867");
        assert_eq!(item.status, TtlPolicyTemplateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_ttl_policy_template("code-491", "label-867");
        if item.move_to(TtlPolicyTemplateStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyTemplateStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_ttl_policy_template("code-491", "label-867");
        item.status = TtlPolicyTemplateStatus::Stopped;
        assert!(
            item.move_to(TtlPolicyTemplateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
