use super::*;

use acme_kit::text;

/// Where a budget burn digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnDigestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BudgetBurnDigestStatus {
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

impl std::fmt::Display for BudgetBurnDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One budget burn digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnDigest {
    pub code: String,
    pub name: String,
    pub ttl_seconds: i64,
    pub summary: String,
    pub title: String,
    pub rate: f64,
    pub status: BudgetBurnDigestStatus,
}

impl BudgetBurnDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnDigestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnDigestStatus) -> bool {
        match self.status {
            BudgetBurnDigestStatus::Idle => {
                return next == BudgetBurnDigestStatus::Warming;
            }
            BudgetBurnDigestStatus::Warming => {
                match next {
                    BudgetBurnDigestStatus::Ready | BudgetBurnDigestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnDigestStatus::Ready => {
                match next {
                    BudgetBurnDigestStatus::Draining | BudgetBurnDigestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnDigestStatus::Draining => {
                match next {
                    BudgetBurnDigestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: BudgetBurnDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "budget burn digest", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.rate + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnDigest {
        BudgetBurnDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            summary: self.summary.clone(),
            title: self.title.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a budget burn digest in status idle.
pub fn new_budget_burn_digest(code: &str, name: &str) -> BudgetBurnDigest {
    BudgetBurnDigest {
        code: code.to_string(),
        name: name.to_string(),
        ttl_seconds: 0,
        summary: "".to_string(),
        title: "".to_string(),
        rate: 0.0,
        status: BudgetBurnDigestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_budget_burn_digest("code-288", "name-995");
        assert_eq!(item.status, BudgetBurnDigestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_budget_burn_digest("code-288", "name-995");
        if item.move_to(BudgetBurnDigestStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnDigestStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_budget_burn_digest("code-288", "name-995");
        item.status = BudgetBurnDigestStatus::Stopped;
        assert!(
            item.move_to(BudgetBurnDigestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
