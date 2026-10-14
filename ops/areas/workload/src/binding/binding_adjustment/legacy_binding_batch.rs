use super::*;

use acme_kit::text;

/// Where a legacy binding batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyBindingBatchStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyBindingBatchStatus {
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

impl std::fmt::Display for LegacyBindingBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy binding batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingBatch {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub quantity: i64,
    pub window_days: i64,
    pub capacity: i64,
    pub source: String,
    pub status: LegacyBindingBatchStatus,
}

impl LegacyBindingBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyBindingBatchStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyBindingBatchStatus) -> bool {
        match self.status {
            LegacyBindingBatchStatus::New => {
                match next {
                    LegacyBindingBatchStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyBindingBatchStatus::Queued => {
                return next == LegacyBindingBatchStatus::Done || next == LegacyBindingBatchStatus::Failed;
            }
            LegacyBindingBatchStatus::Running => {
                match next {
                    LegacyBindingBatchStatus::Done | LegacyBindingBatchStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyBindingBatchStatus::Done => {
                return next == LegacyBindingBatchStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyBindingBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy binding batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.capacity as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyBindingBatch {
        LegacyBindingBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            cipher: self.cipher.clone(),
            quantity: self.quantity,
            window_days: self.window_days,
            capacity: self.capacity,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a legacy binding batch in status new.
pub fn new_legacy_binding_batch(code: &str, name: &str) -> LegacyBindingBatch {
    LegacyBindingBatch {
        code: code.to_string(),
        name: name.to_string(),
        cipher: "".to_string(),
        quantity: 0,
        window_days: 0,
        capacity: 0,
        source: "".to_string(),
        status: LegacyBindingBatchStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_legacy_binding_batch("code-708", "name-331");
        assert_eq!(item.status, LegacyBindingBatchStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_legacy_binding_batch("code-708", "name-331");
        if item.move_to(LegacyBindingBatchStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyBindingBatchStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_legacy_binding_batch("code-708", "name-331");
        item.status = LegacyBindingBatchStatus::Failed;
        assert!(
            item.move_to(LegacyBindingBatchStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
