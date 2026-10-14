use super::*;

use acme_kit::text;

/// Where a canary template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryTemplateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CanaryTemplateStatus {
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

impl std::fmt::Display for CanaryTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a canary template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryTemplate {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub segment: String,
    pub attempts: i64,
    pub retries: i64,
    pub active: bool,
    pub status: CanaryTemplateStatus,
}

impl CanaryTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryTemplateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryTemplateStatus) -> bool {
        match self.status {
            CanaryTemplateStatus::Idle => {
                match next {
                    CanaryTemplateStatus::Warming | CanaryTemplateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryTemplateStatus::Warming => {
                return next == CanaryTemplateStatus::Ready || next == CanaryTemplateStatus::Stopped;
            }
            CanaryTemplateStatus::Ready => {
                return next == CanaryTemplateStatus::Stopped;
            }
            CanaryTemplateStatus::Draining => {
                return next == CanaryTemplateStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.retries as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryTemplate {
        CanaryTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            segment: self.segment.clone(),
            attempts: self.attempts,
            retries: self.retries,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a canary template in status idle.
pub fn new_canary_template(code: &str, label: &str) -> CanaryTemplate {
    CanaryTemplate {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        segment: "".to_string(),
        attempts: 0,
        retries: 0,
        active: false,
        status: CanaryTemplateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_canary_template("code-628", "label-255");
        assert_eq!(item.status, CanaryTemplateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_canary_template("code-628", "label-255");
        if item.move_to(CanaryTemplateStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryTemplateStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_canary_template("code-628", "label-255");
        item.status = CanaryTemplateStatus::Stopped;
        assert!(
            item.move_to(CanaryTemplateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
