use super::*;

use acme_kit::text;

/// Where a batched canary audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedCanaryAuditStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BatchedCanaryAuditStatus {
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

impl std::fmt::Display for BatchedCanaryAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched canary audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCanaryAudit {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub unit: String,
    pub port: i64,
    pub scope: String,
    pub ceiling: f64,
    pub margin: f64,
    pub phone: String,
    pub tags: Vec<String>,
    pub reviewer: String,
    pub status: BatchedCanaryAuditStatus,
}

impl BatchedCanaryAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedCanaryAuditStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedCanaryAuditStatus) -> bool {
        match self.status {
            BatchedCanaryAuditStatus::New => {
                match next {
                    BatchedCanaryAuditStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedCanaryAuditStatus::Queued => {
                match next {
                    BatchedCanaryAuditStatus::Done | BatchedCanaryAuditStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedCanaryAuditStatus::Running => {
                match next {
                    BatchedCanaryAuditStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedCanaryAuditStatus::Done => {
                match next {
                    BatchedCanaryAuditStatus::Failed => {
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

    pub fn move_to(&mut self, next: BatchedCanaryAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched canary audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_total(&self) -> f64 {
        ((self.minimum + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedCanaryAudit {
        BatchedCanaryAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            unit: self.unit.clone(),
            port: self.port,
            scope: self.scope.clone(),
            ceiling: self.ceiling,
            margin: self.margin,
            phone: self.phone.clone(),
            tags: self.tags.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a batched canary audit in status new.
pub fn new_batched_canary_audit(code: &str, label: &str) -> BatchedCanaryAudit {
    BatchedCanaryAudit {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        unit: "".to_string(),
        port: 0,
        scope: "".to_string(),
        ceiling: 0.0,
        margin: 0.0,
        phone: "".to_string(),
        tags: Vec::new(),
        reviewer: "".to_string(),
        status: BatchedCanaryAuditStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_batched_canary_audit("code-862", "label-600");
        assert_eq!(item.status, BatchedCanaryAuditStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_batched_canary_audit("code-862", "label-600");
        if item.move_to(BatchedCanaryAuditStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedCanaryAuditStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_batched_canary_audit("code-862", "label-600");
        item.status = BatchedCanaryAuditStatus::Failed;
        assert!(
            item.move_to(BatchedCanaryAuditStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
