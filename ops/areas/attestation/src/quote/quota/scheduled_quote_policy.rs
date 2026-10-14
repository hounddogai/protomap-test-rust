use super::*;

use acme_kit::text;

/// Where a scheduled quote policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledQuotePolicyStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledQuotePolicyStatus {
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

impl std::fmt::Display for ScheduledQuotePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled quote policy as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuotePolicy {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub ratio: f64,
    pub issuer: String,
    pub status: ScheduledQuotePolicyStatus,
}

impl ScheduledQuotePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledQuotePolicyStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledQuotePolicyStatus) -> bool {
        match self.status {
            ScheduledQuotePolicyStatus::Idle => {
                match next {
                    ScheduledQuotePolicyStatus::Ready | ScheduledQuotePolicyStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledQuotePolicyStatus::Warming => {
                match next {
                    ScheduledQuotePolicyStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledQuotePolicyStatus::Ready => {
                return next == ScheduledQuotePolicyStatus::Draining;
            }
            ScheduledQuotePolicyStatus::Draining => {
                match next {
                    ScheduledQuotePolicyStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ScheduledQuotePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled quote policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.weight * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledQuotePolicy {
        ScheduledQuotePolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            ratio: self.ratio,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled quote policy in status idle.
pub fn new_scheduled_quote_policy(code: &str, name: &str) -> ScheduledQuotePolicy {
    ScheduledQuotePolicy {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        ratio: 0.0,
        issuer: "".to_string(),
        status: ScheduledQuotePolicyStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_quote_policy("code-105", "name-400");
        assert_eq!(item.status, ScheduledQuotePolicyStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_scheduled_quote_policy("code-105", "name-400");
        if item.move_to(ScheduledQuotePolicyStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledQuotePolicyStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_quote_policy("code-105", "name-400");
        item.status = ScheduledQuotePolicyStatus::Stopped;
        assert!(
            item.move_to(ScheduledQuotePolicyStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
