use super::*;

use acme_kit::text;

/// Where a budget burn request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnRequestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BudgetBurnRequestStatus {
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

impl std::fmt::Display for BudgetBurnRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One budget burn request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnRequest {
    pub code: String,
    pub name: String,
    pub rotation_days: i64,
    pub cipher: String,
    pub principal: String,
    pub owner_id: String,
    pub fingerprint: String,
    pub threshold: f64,
    pub scope: String,
    pub status: BudgetBurnRequestStatus,
}

impl BudgetBurnRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnRequestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnRequestStatus) -> bool {
        match self.status {
            BudgetBurnRequestStatus::New => {
                return next == BudgetBurnRequestStatus::Failed;
            }
            BudgetBurnRequestStatus::Queued => {
                return next == BudgetBurnRequestStatus::Failed;
            }
            BudgetBurnRequestStatus::Running => {
                match next {
                    BudgetBurnRequestStatus::Done | BudgetBurnRequestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnRequestStatus::Done => {
                match next {
                    BudgetBurnRequestStatus::Failed => {
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

    pub fn move_to(&mut self, next: BudgetBurnRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "budget burn request", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.rotation_days as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnRequest {
        BudgetBurnRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            cipher: self.cipher.clone(),
            principal: self.principal.clone(),
            owner_id: self.owner_id.clone(),
            fingerprint: self.fingerprint.clone(),
            threshold: self.threshold,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a budget burn request in status new.
pub fn new_budget_burn_request(code: &str, name: &str) -> BudgetBurnRequest {
    BudgetBurnRequest {
        code: code.to_string(),
        name: name.to_string(),
        rotation_days: 0,
        cipher: "".to_string(),
        principal: "".to_string(),
        owner_id: "".to_string(),
        fingerprint: "".to_string(),
        threshold: 0.0,
        scope: "".to_string(),
        status: BudgetBurnRequestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_budget_burn_request("code-777", "name-417");
        assert_eq!(item.status, BudgetBurnRequestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_budget_burn_request("code-777", "name-417");
        if item.move_to(BudgetBurnRequestStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnRequestStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_budget_burn_request("code-777", "name-417");
        item.status = BudgetBurnRequestStatus::Failed;
        assert!(
            item.move_to(BudgetBurnRequestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
