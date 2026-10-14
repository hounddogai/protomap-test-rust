use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a flow log budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogBudgetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FlowLogBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for FlowLogBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogBudget {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub attributes: BTreeMap<String, String>,
    pub country: String,
    pub cost: f64,
    pub limit: i64,
    pub status: FlowLogBudgetStatus,
}

impl FlowLogBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogBudgetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogBudgetStatus) -> bool {
        match self.status {
            FlowLogBudgetStatus::New => {
                match next {
                    FlowLogBudgetStatus::Running | FlowLogBudgetStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogBudgetStatus::Queued => {
                match next {
                    FlowLogBudgetStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogBudgetStatus::Running => {
                match next {
                    FlowLogBudgetStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogBudgetStatus::Done => {
                match next {
                    FlowLogBudgetStatus::Failed => {
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

    pub fn move_to(&mut self, next: FlowLogBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "flow log budget", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.cost - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogBudget {
        FlowLogBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            attributes: self.attributes.clone(),
            country: self.country.clone(),
            cost: self.cost,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a flow log budget in status new.
pub fn new_flow_log_budget(code: &str, name: &str) -> FlowLogBudget {
    FlowLogBudget {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        attributes: BTreeMap::new(),
        country: "".to_string(),
        cost: 0.0,
        limit: 0,
        status: FlowLogBudgetStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_flow_log_budget("code-370", "name-884");
        assert_eq!(item.status, FlowLogBudgetStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_flow_log_budget("code-370", "name-884");
        if item.move_to(FlowLogBudgetStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogBudgetStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_flow_log_budget("code-370", "name-884");
        item.status = FlowLogBudgetStatus::Failed;
        assert!(item.move_to(FlowLogBudgetStatus::New).is_err(), "expected move to fail");
    }
}
