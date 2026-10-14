use super::*;

use acme_kit::text;

/// Where a capped canary rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedCanaryRuleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CappedCanaryRuleStatus {
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

impl std::fmt::Display for CappedCanaryRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped canary rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedCanaryRule {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub ceiling: f64,
    pub description: String,
    pub batch_size: i64,
    pub ratio: f64,
    pub status: CappedCanaryRuleStatus,
}

impl CappedCanaryRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedCanaryRuleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedCanaryRuleStatus) -> bool {
        match self.status {
            CappedCanaryRuleStatus::Idle => {
                match next {
                    CappedCanaryRuleStatus::Ready | CappedCanaryRuleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedCanaryRuleStatus::Warming => {
                match next {
                    CappedCanaryRuleStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedCanaryRuleStatus::Ready => {
                match next {
                    CappedCanaryRuleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedCanaryRuleStatus::Draining => {
                match next {
                    CappedCanaryRuleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: CappedCanaryRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped canary rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.batch_size as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedCanaryRule {
        CappedCanaryRule {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            ceiling: self.ceiling,
            description: self.description.clone(),
            batch_size: self.batch_size,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a capped canary rule in status idle.
pub fn new_capped_canary_rule(code: &str, title: &str) -> CappedCanaryRule {
    CappedCanaryRule {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        ceiling: 0.0,
        description: "".to_string(),
        batch_size: 0,
        ratio: 0.0,
        status: CappedCanaryRuleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_capped_canary_rule("code-460", "title-339");
        assert_eq!(item.status, CappedCanaryRuleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_capped_canary_rule("code-460", "title-339");
        if item.move_to(CappedCanaryRuleStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedCanaryRuleStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_capped_canary_rule("code-460", "title-339");
        item.status = CappedCanaryRuleStatus::Stopped;
        assert!(
            item.move_to(CappedCanaryRuleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
