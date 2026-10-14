use super::*;

use acme_kit::text;

/// Where a overage budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageBudgetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl OverageBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for OverageBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a overage budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageBudget {
    pub code: String,
    pub title: String,
    pub floor_value: f64,
    pub burst: i64,
    pub batch_size: i64,
    pub visible: bool,
    pub retries: i64,
    pub issuer: String,
    pub ends_at: i64,
    pub channel: String,
    pub status: OverageBudgetStatus,
}

impl OverageBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageBudgetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageBudgetStatus) -> bool {
        match self.status {
            OverageBudgetStatus::Requested => {
                match next {
                    OverageBudgetStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageBudgetStatus::Granted => {
                return next == OverageBudgetStatus::Revoked;
            }
            OverageBudgetStatus::Revoked => {
                match next {
                    OverageBudgetStatus::Expired => {
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

    pub fn move_to(&mut self, next: OverageBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.burst as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageBudget {
        OverageBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            floor_value: self.floor_value,
            burst: self.burst,
            batch_size: self.batch_size,
            visible: self.visible,
            retries: self.retries,
            issuer: self.issuer.clone(),
            ends_at: self.ends_at,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a overage budget in status requested.
pub fn new_overage_budget(code: &str, title: &str) -> OverageBudget {
    OverageBudget {
        code: code.to_string(),
        title: title.to_string(),
        floor_value: 0.0,
        burst: 0,
        batch_size: 0,
        visible: false,
        retries: 0,
        issuer: "".to_string(),
        ends_at: 0,
        channel: "".to_string(),
        status: OverageBudgetStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_overage_budget("code-93", "title-501");
        assert_eq!(item.status, OverageBudgetStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_overage_budget("code-93", "title-501");
        if item.move_to(OverageBudgetStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageBudgetStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_overage_budget("code-93", "title-501");
        item.status = OverageBudgetStatus::Expired;
        assert!(
            item.move_to(OverageBudgetStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
