use super::*;

use acme_kit::text;

/// Where a global suspension batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalSuspensionBatchStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GlobalSuspensionBatchStatus {
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

impl std::fmt::Display for GlobalSuspensionBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global suspension batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSuspensionBatch {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub weight: f64,
    pub active: bool,
    pub retries: i64,
    pub maximum: f64,
    pub approved: bool,
    pub status: GlobalSuspensionBatchStatus,
}

impl GlobalSuspensionBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalSuspensionBatchStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalSuspensionBatchStatus) -> bool {
        match self.status {
            GlobalSuspensionBatchStatus::New => {
                return next == GlobalSuspensionBatchStatus::Queued || next == GlobalSuspensionBatchStatus::Failed;
            }
            GlobalSuspensionBatchStatus::Queued => {
                return next == GlobalSuspensionBatchStatus::Running || next == GlobalSuspensionBatchStatus::Failed;
            }
            GlobalSuspensionBatchStatus::Running => {
                match next {
                    GlobalSuspensionBatchStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalSuspensionBatchStatus::Done => {
                return next == GlobalSuspensionBatchStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalSuspensionBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global suspension batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.weight - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalSuspensionBatch {
        GlobalSuspensionBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            weight: self.weight,
            active: self.active,
            retries: self.retries,
            maximum: self.maximum,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global suspension batch in status new.
pub fn new_global_suspension_batch(code: &str, label: &str) -> GlobalSuspensionBatch {
    GlobalSuspensionBatch {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        weight: 0.0,
        active: false,
        retries: 0,
        maximum: 0.0,
        approved: false,
        status: GlobalSuspensionBatchStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_global_suspension_batch("code-633", "label-28");
        assert_eq!(item.status, GlobalSuspensionBatchStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_global_suspension_batch("code-633", "label-28");
        if item.move_to(GlobalSuspensionBatchStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalSuspensionBatchStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_global_suspension_batch("code-633", "label-28");
        item.status = GlobalSuspensionBatchStatus::Failed;
        assert!(
            item.move_to(GlobalSuspensionBatchStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
