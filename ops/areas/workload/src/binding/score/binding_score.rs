use super::*;

use acme_kit::text;

/// Where a binding score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingScoreStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BindingScoreStatus {
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

impl std::fmt::Display for BindingScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One binding score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingScore {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub quota_units: i64,
    pub rate: f64,
    pub window_days: i64,
    pub channel: String,
    pub status: BindingScoreStatus,
}

impl BindingScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingScoreStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingScoreStatus) -> bool {
        match self.status {
            BindingScoreStatus::Idle => {
                return next == BindingScoreStatus::Stopped;
            }
            BindingScoreStatus::Warming => {
                return next == BindingScoreStatus::Ready;
            }
            BindingScoreStatus::Ready => {
                match next {
                    BindingScoreStatus::Draining | BindingScoreStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingScoreStatus::Draining => {
                return next == BindingScoreStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BindingScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "binding score", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.limit as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingScore {
        BindingScore {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            quota_units: self.quota_units,
            rate: self.rate,
            window_days: self.window_days,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a binding score in status idle.
pub fn new_binding_score(code: &str, label: &str) -> BindingScore {
    BindingScore {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        quota_units: 0,
        rate: 0.0,
        window_days: 0,
        channel: "".to_string(),
        status: BindingScoreStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_binding_score("code-434", "label-574");
        assert_eq!(item.status, BindingScoreStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_binding_score("code-434", "label-574");
        if item.move_to(BindingScoreStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingScoreStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_binding_score("code-434", "label-574");
        item.status = BindingScoreStatus::Stopped;
        assert!(item.move_to(BindingScoreStatus::Idle).is_err(), "expected move to fail");
    }
}
