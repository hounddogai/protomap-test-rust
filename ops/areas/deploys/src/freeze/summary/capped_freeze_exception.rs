use super::*;

use acme_kit::text;

/// Where a capped freeze exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedFreezeExceptionStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CappedFreezeExceptionStatus {
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

impl std::fmt::Display for CappedFreezeExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped freeze exception as the freeze team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedFreezeException {
    pub code: String,
    pub label: String,
    pub description: String,
    pub external_ref: String,
    pub capacity: i64,
    pub status: CappedFreezeExceptionStatus,
}

impl CappedFreezeException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedFreezeExceptionStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedFreezeExceptionStatus) -> bool {
        match self.status {
            CappedFreezeExceptionStatus::New => {
                return next == CappedFreezeExceptionStatus::Queued || next == CappedFreezeExceptionStatus::Running;
            }
            CappedFreezeExceptionStatus::Queued => {
                match next {
                    CappedFreezeExceptionStatus::Running | CappedFreezeExceptionStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedFreezeExceptionStatus::Running => {
                match next {
                    CappedFreezeExceptionStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedFreezeExceptionStatus::Done => {
                return next == CappedFreezeExceptionStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedFreezeExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped freeze exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.capacity as f64
    }

    pub fn copy(&self) -> CappedFreezeException {
        CappedFreezeException {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            external_ref: self.external_ref.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a capped freeze exception in status new.
pub fn new_capped_freeze_exception(code: &str, label: &str) -> CappedFreezeException {
    CappedFreezeException {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        external_ref: "".to_string(),
        capacity: 0,
        status: CappedFreezeExceptionStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_capped_freeze_exception("code-911", "label-599");
        assert_eq!(item.status, CappedFreezeExceptionStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_capped_freeze_exception("code-911", "label-599");
        if item.move_to(CappedFreezeExceptionStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedFreezeExceptionStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_capped_freeze_exception("code-911", "label-599");
        item.status = CappedFreezeExceptionStatus::Failed;
        assert!(
            item.move_to(CappedFreezeExceptionStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
