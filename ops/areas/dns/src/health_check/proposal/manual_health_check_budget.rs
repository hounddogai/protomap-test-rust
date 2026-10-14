use super::*;

use acme_kit::text;

/// Where a manual health check budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualHealthCheckBudgetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualHealthCheckBudgetStatus {
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

impl std::fmt::Display for ManualHealthCheckBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual health check budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualHealthCheckBudget {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub reviewer: String,
    pub quantity: i64,
    pub updated_at: i64,
    pub status: ManualHealthCheckBudgetStatus,
}

impl ManualHealthCheckBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualHealthCheckBudgetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualHealthCheckBudgetStatus) -> bool {
        match self.status {
            ManualHealthCheckBudgetStatus::New => {
                return next == ManualHealthCheckBudgetStatus::Queued || next == ManualHealthCheckBudgetStatus::Running;
            }
            ManualHealthCheckBudgetStatus::Queued => {
                match next {
                    ManualHealthCheckBudgetStatus::Done | ManualHealthCheckBudgetStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualHealthCheckBudgetStatus::Running => {
                match next {
                    ManualHealthCheckBudgetStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualHealthCheckBudgetStatus::Done => {
                return next == ManualHealthCheckBudgetStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualHealthCheckBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual health check budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.updated_at as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualHealthCheckBudget {
        ManualHealthCheckBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            reviewer: self.reviewer.clone(),
            quantity: self.quantity,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual health check budget in status new.
pub fn new_manual_health_check_budget(code: &str, title: &str) -> ManualHealthCheckBudget {
    ManualHealthCheckBudget {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        reviewer: "".to_string(),
        quantity: 0,
        updated_at: 0,
        status: ManualHealthCheckBudgetStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_manual_health_check_budget("code-735", "title-461");
        assert_eq!(item.status, ManualHealthCheckBudgetStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_manual_health_check_budget("code-735", "title-461");
        if item.move_to(ManualHealthCheckBudgetStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualHealthCheckBudgetStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_manual_health_check_budget("code-735", "title-461");
        item.status = ManualHealthCheckBudgetStatus::Failed;
        assert!(
            item.move_to(ManualHealthCheckBudgetStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
