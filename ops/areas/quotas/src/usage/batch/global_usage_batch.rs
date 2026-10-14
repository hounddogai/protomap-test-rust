use super::*;

use acme_kit::text;

/// Where a global usage batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalUsageBatchStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GlobalUsageBatchStatus {
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

impl std::fmt::Display for GlobalUsageBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global usage batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUsageBatch {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub cost: f64,
    pub description: String,
    pub attempts: i64,
    pub budget: f64,
    pub status: GlobalUsageBatchStatus,
}

impl GlobalUsageBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalUsageBatchStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalUsageBatchStatus) -> bool {
        match self.status {
            GlobalUsageBatchStatus::New => {
                return next == GlobalUsageBatchStatus::Running;
            }
            GlobalUsageBatchStatus::Queued => {
                match next {
                    GlobalUsageBatchStatus::Running | GlobalUsageBatchStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalUsageBatchStatus::Running => {
                match next {
                    GlobalUsageBatchStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalUsageBatchStatus::Done => {
                match next {
                    GlobalUsageBatchStatus::Failed => {
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

    pub fn move_to(&mut self, next: GlobalUsageBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global usage batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.attempts as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalUsageBatch {
        GlobalUsageBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            cost: self.cost,
            description: self.description.clone(),
            attempts: self.attempts,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global usage batch in status new.
pub fn new_global_usage_batch(code: &str, label: &str) -> GlobalUsageBatch {
    GlobalUsageBatch {
        code: code.to_string(),
        label: label.to_string(),
        contact_email: "".to_string(),
        cost: 0.0,
        description: "".to_string(),
        attempts: 0,
        budget: 0.0,
        status: GlobalUsageBatchStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_global_usage_batch("code-145", "label-93");
        assert_eq!(item.status, GlobalUsageBatchStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_global_usage_batch("code-145", "label-93");
        if item.move_to(GlobalUsageBatchStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalUsageBatchStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_global_usage_batch("code-145", "label-93");
        item.status = GlobalUsageBatchStatus::Failed;
        assert!(
            item.move_to(GlobalUsageBatchStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
