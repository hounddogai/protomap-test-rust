use super::*;

use acme_kit::text;

/// Where a scheduled quote record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledQuoteRecordStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ScheduledQuoteRecordStatus {
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

impl std::fmt::Display for ScheduledQuoteRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled quote record as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuoteRecord {
    pub code: String,
    pub title: String,
    pub contact_email: String,
    pub external_ref: String,
    pub minimum: f64,
    pub status: ScheduledQuoteRecordStatus,
}

impl ScheduledQuoteRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledQuoteRecordStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledQuoteRecordStatus) -> bool {
        match self.status {
            ScheduledQuoteRecordStatus::New => {
                match next {
                    ScheduledQuoteRecordStatus::Queued | ScheduledQuoteRecordStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledQuoteRecordStatus::Queued => {
                return next == ScheduledQuoteRecordStatus::Failed;
            }
            ScheduledQuoteRecordStatus::Running => {
                return next == ScheduledQuoteRecordStatus::Failed;
            }
            ScheduledQuoteRecordStatus::Done => {
                match next {
                    ScheduledQuoteRecordStatus::Failed => {
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

    pub fn move_to(&mut self, next: ScheduledQuoteRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled quote record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.minimum
    }

    pub fn copy(&self) -> ScheduledQuoteRecord {
        ScheduledQuoteRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            external_ref: self.external_ref.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a scheduled quote record in status new.
pub fn new_scheduled_quote_record(code: &str, title: &str) -> ScheduledQuoteRecord {
    ScheduledQuoteRecord {
        code: code.to_string(),
        title: title.to_string(),
        contact_email: "".to_string(),
        external_ref: "".to_string(),
        minimum: 0.0,
        status: ScheduledQuoteRecordStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_scheduled_quote_record("code-662", "title-471");
        assert_eq!(item.status, ScheduledQuoteRecordStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_scheduled_quote_record("code-662", "title-471");
        if item.move_to(ScheduledQuoteRecordStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledQuoteRecordStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_scheduled_quote_record("code-662", "title-471");
        item.status = ScheduledQuoteRecordStatus::Failed;
        assert!(
            item.move_to(ScheduledQuoteRecordStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
