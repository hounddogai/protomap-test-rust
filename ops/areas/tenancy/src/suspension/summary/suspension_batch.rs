use super::*;

use acme_kit::text;

/// Where a suspension batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionBatchStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SuspensionBatchStatus {
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

impl std::fmt::Display for SuspensionBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One suspension batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionBatch {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub label: String,
    pub unit: String,
    pub issuer: String,
    pub created_at: i64,
    pub scope: String,
    pub checksum: String,
    pub key_id: String,
    pub quantity: i64,
    pub status: SuspensionBatchStatus,
}

impl SuspensionBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionBatchStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionBatchStatus) -> bool {
        match self.status {
            SuspensionBatchStatus::New => {
                return next == SuspensionBatchStatus::Queued;
            }
            SuspensionBatchStatus::Queued => {
                match next {
                    SuspensionBatchStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionBatchStatus::Running => {
                match next {
                    SuspensionBatchStatus::Done | SuspensionBatchStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionBatchStatus::Done => {
                return next == SuspensionBatchStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.budget * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionBatch {
        SuspensionBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            label: self.label.clone(),
            unit: self.unit.clone(),
            issuer: self.issuer.clone(),
            created_at: self.created_at,
            scope: self.scope.clone(),
            checksum: self.checksum.clone(),
            key_id: self.key_id.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a suspension batch in status new.
pub fn new_suspension_batch(code: &str, name: &str) -> SuspensionBatch {
    SuspensionBatch {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        label: "".to_string(),
        unit: "".to_string(),
        issuer: "".to_string(),
        created_at: 0,
        scope: "".to_string(),
        checksum: "".to_string(),
        key_id: "".to_string(),
        quantity: 0,
        status: SuspensionBatchStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_suspension_batch("code-763", "name-953");
        assert_eq!(item.status, SuspensionBatchStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_suspension_batch("code-763", "name-953");
        if item.move_to(SuspensionBatchStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionBatchStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_suspension_batch("code-763", "name-953");
        item.status = SuspensionBatchStatus::Failed;
        assert!(
            item.move_to(SuspensionBatchStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
