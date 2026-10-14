use super::*;

use acme_kit::text;

/// Where a dns record budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordBudgetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DnsRecordBudgetStatus {
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

impl std::fmt::Display for DnsRecordBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dns record budget as the dns record team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordBudget {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub host: String,
    pub channel: String,
    pub amount: f64,
    pub status: DnsRecordBudgetStatus,
}

impl DnsRecordBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordBudgetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordBudgetStatus) -> bool {
        match self.status {
            DnsRecordBudgetStatus::Idle => {
                return next == DnsRecordBudgetStatus::Ready;
            }
            DnsRecordBudgetStatus::Warming => {
                match next {
                    DnsRecordBudgetStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordBudgetStatus::Ready => {
                return next == DnsRecordBudgetStatus::Stopped;
            }
            DnsRecordBudgetStatus::Draining => {
                match next {
                    DnsRecordBudgetStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DnsRecordBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "dns record budget", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        (self.amount * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordBudget {
        DnsRecordBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            host: self.host.clone(),
            channel: self.channel.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a dns record budget in status idle.
pub fn new_dns_record_budget(code: &str, name: &str) -> DnsRecordBudget {
    DnsRecordBudget {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        host: "".to_string(),
        channel: "".to_string(),
        amount: 0.0,
        status: DnsRecordBudgetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_dns_record_budget("code-517", "name-305");
        assert_eq!(item.status, DnsRecordBudgetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_dns_record_budget("code-517", "name-305");
        if item.move_to(DnsRecordBudgetStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordBudgetStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_dns_record_budget("code-517", "name-305");
        item.status = DnsRecordBudgetStatus::Stopped;
        assert!(
            item.move_to(DnsRecordBudgetStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
