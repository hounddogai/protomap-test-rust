use super::*;

use acme_kit::text;

/// Where a budget burn record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnRecordStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BudgetBurnRecordStatus {
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

impl std::fmt::Display for BudgetBurnRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnRecord {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub host: String,
    pub notes: String,
    pub burst: i64,
    pub revision: i64,
    pub sequence: i64,
    pub threshold: f64,
    pub retries: i64,
    pub status: BudgetBurnRecordStatus,
}

impl BudgetBurnRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnRecordStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnRecordStatus) -> bool {
        match self.status {
            BudgetBurnRecordStatus::Draft => {
                return next == BudgetBurnRecordStatus::Closed;
            }
            BudgetBurnRecordStatus::Pending => {
                return next == BudgetBurnRecordStatus::Suspended || next == BudgetBurnRecordStatus::Closed;
            }
            BudgetBurnRecordStatus::Active => {
                return next == BudgetBurnRecordStatus::Suspended;
            }
            BudgetBurnRecordStatus::Suspended => {
                return next == BudgetBurnRecordStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BudgetBurnRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.burst as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnRecord {
        BudgetBurnRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            host: self.host.clone(),
            notes: self.notes.clone(),
            burst: self.burst,
            revision: self.revision,
            sequence: self.sequence,
            threshold: self.threshold,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a budget burn record in status draft.
pub fn new_budget_burn_record(code: &str, title: &str) -> BudgetBurnRecord {
    BudgetBurnRecord {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        host: "".to_string(),
        notes: "".to_string(),
        burst: 0,
        revision: 0,
        sequence: 0,
        threshold: 0.0,
        retries: 0,
        status: BudgetBurnRecordStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_budget_burn_record("code-818", "title-336");
        assert_eq!(item.status, BudgetBurnRecordStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_budget_burn_record("code-818", "title-336");
        if item.move_to(BudgetBurnRecordStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnRecordStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_budget_burn_record("code-818", "title-336");
        item.status = BudgetBurnRecordStatus::Closed;
        assert!(
            item.move_to(BudgetBurnRecordStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
