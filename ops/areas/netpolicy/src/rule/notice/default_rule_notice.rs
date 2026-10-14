use super::*;

use acme_kit::text;

/// Where a default rule notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRuleNoticeStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultRuleNoticeStatus {
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

impl std::fmt::Display for DefaultRuleNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default rule notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRuleNotice {
    pub code: String,
    pub label: String,
    pub host: String,
    pub quantity: i64,
    pub ratio: f64,
    pub notes: String,
    pub reviewer: String,
    pub grace_days: i64,
    pub cost: f64,
    pub window_days: i64,
    pub status: DefaultRuleNoticeStatus,
}

impl DefaultRuleNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRuleNoticeStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRuleNoticeStatus) -> bool {
        match self.status {
            DefaultRuleNoticeStatus::Idle => {
                match next {
                    DefaultRuleNoticeStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRuleNoticeStatus::Warming => {
                return next == DefaultRuleNoticeStatus::Ready;
            }
            DefaultRuleNoticeStatus::Ready => {
                match next {
                    DefaultRuleNoticeStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRuleNoticeStatus::Draining => {
                match next {
                    DefaultRuleNoticeStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DefaultRuleNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default rule notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "default rule notice", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.ratio * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultRuleNotice {
        DefaultRuleNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            host: self.host.clone(),
            quantity: self.quantity,
            ratio: self.ratio,
            notes: self.notes.clone(),
            reviewer: self.reviewer.clone(),
            grace_days: self.grace_days,
            cost: self.cost,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a default rule notice in status idle.
pub fn new_default_rule_notice(code: &str, label: &str) -> DefaultRuleNotice {
    DefaultRuleNotice {
        code: code.to_string(),
        label: label.to_string(),
        host: "".to_string(),
        quantity: 0,
        ratio: 0.0,
        notes: "".to_string(),
        reviewer: "".to_string(),
        grace_days: 0,
        cost: 0.0,
        window_days: 0,
        status: DefaultRuleNoticeStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_default_rule_notice("code-489", "label-312");
        assert_eq!(item.status, DefaultRuleNoticeStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_default_rule_notice("code-489", "label-312");
        if item.move_to(DefaultRuleNoticeStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultRuleNoticeStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_default_rule_notice("code-489", "label-312");
        item.status = DefaultRuleNoticeStatus::Stopped;
        assert!(
            item.move_to(DefaultRuleNoticeStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
