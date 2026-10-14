use super::*;

use acme_kit::text;

/// Where a manual quote batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualQuoteBatchStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualQuoteBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ManualQuoteBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual quote batch as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuoteBatch {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub namespace: String,
    pub ceiling: f64,
    pub retries: i64,
    pub notes: String,
    pub category: String,
    pub attempts: i64,
    pub threshold: f64,
    pub external_ref: String,
    pub status: ManualQuoteBatchStatus,
}

impl ManualQuoteBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualQuoteBatchStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualQuoteBatchStatus) -> bool {
        match self.status {
            ManualQuoteBatchStatus::Idle => {
                return next == ManualQuoteBatchStatus::Draining || next == ManualQuoteBatchStatus::Stopped;
            }
            ManualQuoteBatchStatus::Warming => {
                return next == ManualQuoteBatchStatus::Draining;
            }
            ManualQuoteBatchStatus::Ready => {
                match next {
                    ManualQuoteBatchStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualQuoteBatchStatus::Draining => {
                return next == ManualQuoteBatchStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualQuoteBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual quote batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual quote batch", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.rotation_days as f64 + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualQuoteBatch {
        ManualQuoteBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            namespace: self.namespace.clone(),
            ceiling: self.ceiling,
            retries: self.retries,
            notes: self.notes.clone(),
            category: self.category.clone(),
            attempts: self.attempts,
            threshold: self.threshold,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a manual quote batch in status idle.
pub fn new_manual_quote_batch(code: &str, title: &str) -> ManualQuoteBatch {
    ManualQuoteBatch {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        namespace: "".to_string(),
        ceiling: 0.0,
        retries: 0,
        notes: "".to_string(),
        category: "".to_string(),
        attempts: 0,
        threshold: 0.0,
        external_ref: "".to_string(),
        status: ManualQuoteBatchStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_manual_quote_batch("code-798", "title-379");
        assert_eq!(item.status, ManualQuoteBatchStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_manual_quote_batch("code-798", "title-379");
        if item.move_to(ManualQuoteBatchStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualQuoteBatchStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_manual_quote_batch("code-798", "title-379");
        item.status = ManualQuoteBatchStatus::Stopped;
        assert!(
            item.move_to(ManualQuoteBatchStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
