use super::*;

use acme_kit::text;

/// Where a seasonal action item batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalActionItemBatchStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SeasonalActionItemBatchStatus {
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

impl std::fmt::Display for SeasonalActionItemBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal action item batch as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemBatch {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub port: i64,
    pub grace_days: i64,
    pub retries: i64,
    pub quota_units: i64,
    pub currency: String,
    pub tenant_id: String,
    pub status: SeasonalActionItemBatchStatus,
}

impl SeasonalActionItemBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalActionItemBatchStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalActionItemBatchStatus) -> bool {
        match self.status {
            SeasonalActionItemBatchStatus::New => {
                return next == SeasonalActionItemBatchStatus::Queued;
            }
            SeasonalActionItemBatchStatus::Queued => {
                return next == SeasonalActionItemBatchStatus::Running || next == SeasonalActionItemBatchStatus::Failed;
            }
            SeasonalActionItemBatchStatus::Running => {
                return next == SeasonalActionItemBatchStatus::Done || next == SeasonalActionItemBatchStatus::Failed;
            }
            SeasonalActionItemBatchStatus::Done => {
                return next == SeasonalActionItemBatchStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalActionItemBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal action item batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal action item batch", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.port as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalActionItemBatch {
        SeasonalActionItemBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            port: self.port,
            grace_days: self.grace_days,
            retries: self.retries,
            quota_units: self.quota_units,
            currency: self.currency.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a seasonal action item batch in status new.
pub fn new_seasonal_action_item_batch(code: &str, title: &str) -> SeasonalActionItemBatch {
    SeasonalActionItemBatch {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        port: 0,
        grace_days: 0,
        retries: 0,
        quota_units: 0,
        currency: "".to_string(),
        tenant_id: "".to_string(),
        status: SeasonalActionItemBatchStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_seasonal_action_item_batch("code-196", "title-845");
        assert_eq!(item.status, SeasonalActionItemBatchStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_seasonal_action_item_batch("code-196", "title-845");
        if item.move_to(SeasonalActionItemBatchStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalActionItemBatchStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_seasonal_action_item_batch("code-196", "title-845");
        item.status = SeasonalActionItemBatchStatus::Failed;
        assert!(
            item.move_to(SeasonalActionItemBatchStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
