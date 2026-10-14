use super::*;

use acme_kit::text;

/// Where a budget burn draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnDraftStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BudgetBurnDraftStatus {
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

impl std::fmt::Display for BudgetBurnDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a budget burn draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnDraft {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub description: String,
    pub unit: String,
    pub expires_at: i64,
    pub updated_at: i64,
    pub status: BudgetBurnDraftStatus,
}

impl BudgetBurnDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnDraftStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnDraftStatus) -> bool {
        match self.status {
            BudgetBurnDraftStatus::New => {
                return next == BudgetBurnDraftStatus::Running;
            }
            BudgetBurnDraftStatus::Queued => {
                return next == BudgetBurnDraftStatus::Running || next == BudgetBurnDraftStatus::Done;
            }
            BudgetBurnDraftStatus::Running => {
                match next {
                    BudgetBurnDraftStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnDraftStatus::Done => {
                return next == BudgetBurnDraftStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BudgetBurnDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.updated_at as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnDraft {
        BudgetBurnDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            description: self.description.clone(),
            unit: self.unit.clone(),
            expires_at: self.expires_at,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a budget burn draft in status new.
pub fn new_budget_burn_draft(code: &str, name: &str) -> BudgetBurnDraft {
    BudgetBurnDraft {
        code: code.to_string(),
        name: name.to_string(),
        locale: "".to_string(),
        description: "".to_string(),
        unit: "".to_string(),
        expires_at: 0,
        updated_at: 0,
        status: BudgetBurnDraftStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_budget_burn_draft("code-321", "name-819");
        assert_eq!(item.status, BudgetBurnDraftStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_budget_burn_draft("code-321", "name-819");
        if item.move_to(BudgetBurnDraftStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnDraftStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_budget_burn_draft("code-321", "name-819");
        item.status = BudgetBurnDraftStatus::Failed;
        assert!(
            item.move_to(BudgetBurnDraftStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
