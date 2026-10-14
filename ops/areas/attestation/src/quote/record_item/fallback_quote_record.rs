use super::*;

use acme_kit::text;

/// Where a fallback quote record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackQuoteRecordStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackQuoteRecordStatus {
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

impl std::fmt::Display for FallbackQuoteRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback quote record and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteRecord {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub archived: bool,
    pub grace_days: i64,
    pub key_id: String,
    pub status: FallbackQuoteRecordStatus,
}

impl FallbackQuoteRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackQuoteRecordStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackQuoteRecordStatus) -> bool {
        match self.status {
            FallbackQuoteRecordStatus::New => {
                return next == FallbackQuoteRecordStatus::Failed;
            }
            FallbackQuoteRecordStatus::Queued => {
                return next == FallbackQuoteRecordStatus::Running || next == FallbackQuoteRecordStatus::Failed;
            }
            FallbackQuoteRecordStatus::Running => {
                match next {
                    FallbackQuoteRecordStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackQuoteRecordStatus::Done => {
                return next == FallbackQuoteRecordStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackQuoteRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback quote record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> FallbackQuoteRecord {
        FallbackQuoteRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            archived: self.archived,
            grace_days: self.grace_days,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback quote record in status new.
pub fn new_fallback_quote_record(code: &str, name: &str) -> FallbackQuoteRecord {
    FallbackQuoteRecord {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        archived: false,
        grace_days: 0,
        key_id: "".to_string(),
        status: FallbackQuoteRecordStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_fallback_quote_record("code-841", "name-904");
        assert_eq!(item.status, FallbackQuoteRecordStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_fallback_quote_record("code-841", "name-904");
        if item.move_to(FallbackQuoteRecordStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackQuoteRecordStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_fallback_quote_record("code-841", "name-904");
        item.status = FallbackQuoteRecordStatus::Failed;
        assert!(
            item.move_to(FallbackQuoteRecordStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
