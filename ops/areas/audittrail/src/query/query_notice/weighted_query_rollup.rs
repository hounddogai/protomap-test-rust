use super::*;

use acme_kit::text;

/// Where a weighted query rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedQueryRollupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl WeightedQueryRollupStatus {
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

impl std::fmt::Display for WeightedQueryRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted query rollup as the query team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQueryRollup {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub pinned: bool,
    pub discount: f64,
    pub amount: f64,
    pub status: WeightedQueryRollupStatus,
}

impl WeightedQueryRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedQueryRollupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedQueryRollupStatus) -> bool {
        match self.status {
            WeightedQueryRollupStatus::New => {
                return next == WeightedQueryRollupStatus::Done || next == WeightedQueryRollupStatus::Failed;
            }
            WeightedQueryRollupStatus::Queued => {
                return next == WeightedQueryRollupStatus::Failed;
            }
            WeightedQueryRollupStatus::Running => {
                return next == WeightedQueryRollupStatus::Done;
            }
            WeightedQueryRollupStatus::Done => {
                return next == WeightedQueryRollupStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedQueryRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted query rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.discount * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedQueryRollup {
        WeightedQueryRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            pinned: self.pinned,
            discount: self.discount,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a weighted query rollup in status new.
pub fn new_weighted_query_rollup(code: &str, title: &str) -> WeightedQueryRollup {
    WeightedQueryRollup {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        pinned: false,
        discount: 0.0,
        amount: 0.0,
        status: WeightedQueryRollupStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_weighted_query_rollup("code-181", "title-987");
        assert_eq!(item.status, WeightedQueryRollupStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_weighted_query_rollup("code-181", "title-987");
        if item.move_to(WeightedQueryRollupStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedQueryRollupStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_weighted_query_rollup("code-181", "title-987");
        item.status = WeightedQueryRollupStatus::Failed;
        assert!(
            item.move_to(WeightedQueryRollupStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
