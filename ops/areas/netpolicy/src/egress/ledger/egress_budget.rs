use super::*;

use acme_kit::text;

/// Where a egress budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressBudgetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EgressBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for EgressBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One egress budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressBudget {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub ends_at: i64,
    pub ratio: f64,
    pub visible: bool,
    pub channel: String,
    pub quota_units: i64,
    pub status: EgressBudgetStatus,
}

impl EgressBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressBudgetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressBudgetStatus) -> bool {
        match self.status {
            EgressBudgetStatus::Draft => {
                match next {
                    EgressBudgetStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressBudgetStatus::Pending => {
                return next == EgressBudgetStatus::Suspended || next == EgressBudgetStatus::Closed;
            }
            EgressBudgetStatus::Active => {
                match next {
                    EgressBudgetStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressBudgetStatus::Suspended => {
                match next {
                    EgressBudgetStatus::Closed => {
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

    pub fn move_to(&mut self, next: EgressBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress budget {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.ratio + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressBudget {
        EgressBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            ends_at: self.ends_at,
            ratio: self.ratio,
            visible: self.visible,
            channel: self.channel.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a egress budget in status draft.
pub fn new_egress_budget(code: &str, title: &str) -> EgressBudget {
    EgressBudget {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        ends_at: 0,
        ratio: 0.0,
        visible: false,
        channel: "".to_string(),
        quota_units: 0,
        status: EgressBudgetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_egress_budget("code-707", "title-589");
        assert_eq!(item.status, EgressBudgetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_egress_budget("code-707", "title-589");
        if item.move_to(EgressBudgetStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressBudgetStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_egress_budget("code-707", "title-589");
        item.status = EgressBudgetStatus::Closed;
        assert!(
            item.move_to(EgressBudgetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
