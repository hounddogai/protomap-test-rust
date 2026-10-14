use super::*;

use acme_kit::text;

/// Where a default rule cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRuleCycleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultRuleCycleStatus {
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

impl std::fmt::Display for DefaultRuleCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default rule cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRuleCycle {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub fingerprint: String,
    pub minimum: f64,
    pub segment: String,
    pub timezone: String,
    pub status: DefaultRuleCycleStatus,
}

impl DefaultRuleCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRuleCycleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRuleCycleStatus) -> bool {
        match self.status {
            DefaultRuleCycleStatus::Idle => {
                return next == DefaultRuleCycleStatus::Ready;
            }
            DefaultRuleCycleStatus::Warming => {
                match next {
                    DefaultRuleCycleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRuleCycleStatus::Ready => {
                return next == DefaultRuleCycleStatus::Stopped;
            }
            DefaultRuleCycleStatus::Draining => {
                match next {
                    DefaultRuleCycleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DefaultRuleCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default rule cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default rule cycle", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.minimum - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultRuleCycle {
        DefaultRuleCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            fingerprint: self.fingerprint.clone(),
            minimum: self.minimum,
            segment: self.segment.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a default rule cycle in status idle.
pub fn new_default_rule_cycle(code: &str, name: &str) -> DefaultRuleCycle {
    DefaultRuleCycle {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        fingerprint: "".to_string(),
        minimum: 0.0,
        segment: "".to_string(),
        timezone: "".to_string(),
        status: DefaultRuleCycleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_default_rule_cycle("code-68", "name-423");
        assert_eq!(item.status, DefaultRuleCycleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_default_rule_cycle("code-68", "name-423");
        if item.move_to(DefaultRuleCycleStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultRuleCycleStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_default_rule_cycle("code-68", "name-423");
        item.status = DefaultRuleCycleStatus::Stopped;
        assert!(
            item.move_to(DefaultRuleCycleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
