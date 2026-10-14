use super::*;

use acme_kit::text;

/// Where a isolation policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationPolicyStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl IsolationPolicyStatus {
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

impl std::fmt::Display for IsolationPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A isolation policy as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationPolicy {
    pub code: String,
    pub title: String,
    pub region: String,
    pub rate: f64,
    pub batch_size: i64,
    pub score: f64,
    pub port: i64,
    pub quantity: i64,
    pub ratio: f64,
    pub status: IsolationPolicyStatus,
}

impl IsolationPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationPolicyStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationPolicyStatus) -> bool {
        match self.status {
            IsolationPolicyStatus::Idle => {
                match next {
                    IsolationPolicyStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationPolicyStatus::Warming => {
                return next == IsolationPolicyStatus::Stopped;
            }
            IsolationPolicyStatus::Ready => {
                return next == IsolationPolicyStatus::Draining || next == IsolationPolicyStatus::Stopped;
            }
            IsolationPolicyStatus::Draining => {
                match next {
                    IsolationPolicyStatus::Stopped => {
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

    pub fn move_to(&mut self, next: IsolationPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "isolation policy", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.quantity as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationPolicy {
        IsolationPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            rate: self.rate,
            batch_size: self.batch_size,
            score: self.score,
            port: self.port,
            quantity: self.quantity,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a isolation policy in status idle.
pub fn new_isolation_policy(code: &str, title: &str) -> IsolationPolicy {
    IsolationPolicy {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        rate: 0.0,
        batch_size: 0,
        score: 0.0,
        port: 0,
        quantity: 0,
        ratio: 0.0,
        status: IsolationPolicyStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_isolation_policy("code-217", "title-308");
        assert_eq!(item.status, IsolationPolicyStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_isolation_policy("code-217", "title-308");
        if item.move_to(IsolationPolicyStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationPolicyStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_isolation_policy("code-217", "title-308");
        item.status = IsolationPolicyStatus::Stopped;
        assert!(
            item.move_to(IsolationPolicyStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
