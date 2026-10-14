use super::*;

use acme_kit::text;

/// Where a tiered failover batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredFailoverBatchStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TieredFailoverBatchStatus {
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

impl std::fmt::Display for TieredFailoverBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered failover batch as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverBatch {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub rate: f64,
    pub owner_id: String,
    pub currency: String,
    pub duration_minutes: i64,
    pub key_id: String,
    pub burst: i64,
    pub amount: f64,
    pub status: TieredFailoverBatchStatus,
}

impl TieredFailoverBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredFailoverBatchStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredFailoverBatchStatus) -> bool {
        match self.status {
            TieredFailoverBatchStatus::New => {
                return next == TieredFailoverBatchStatus::Queued;
            }
            TieredFailoverBatchStatus::Queued => {
                match next {
                    TieredFailoverBatchStatus::Running | TieredFailoverBatchStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredFailoverBatchStatus::Running => {
                return next == TieredFailoverBatchStatus::Done || next == TieredFailoverBatchStatus::Failed;
            }
            TieredFailoverBatchStatus::Done => {
                return next == TieredFailoverBatchStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredFailoverBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered failover batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered failover batch", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.burst as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredFailoverBatch {
        TieredFailoverBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            rate: self.rate,
            owner_id: self.owner_id.clone(),
            currency: self.currency.clone(),
            duration_minutes: self.duration_minutes,
            key_id: self.key_id.clone(),
            burst: self.burst,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tiered failover batch in status new.
pub fn new_tiered_failover_batch(code: &str, label: &str) -> TieredFailoverBatch {
    TieredFailoverBatch {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        rate: 0.0,
        owner_id: "".to_string(),
        currency: "".to_string(),
        duration_minutes: 0,
        key_id: "".to_string(),
        burst: 0,
        amount: 0.0,
        status: TieredFailoverBatchStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_tiered_failover_batch("code-169", "label-505");
        assert_eq!(item.status, TieredFailoverBatchStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_tiered_failover_batch("code-169", "label-505");
        if item.move_to(TieredFailoverBatchStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredFailoverBatchStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_tiered_failover_batch("code-169", "label-505");
        item.status = TieredFailoverBatchStatus::Failed;
        assert!(
            item.move_to(TieredFailoverBatchStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
