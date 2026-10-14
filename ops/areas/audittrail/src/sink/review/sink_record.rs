use super::*;

use acme_kit::text;

/// Where a sink record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkRecordStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SinkRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for SinkRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One sink record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkRecord {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub active: bool,
    pub threshold: f64,
    pub status: SinkRecordStatus,
}

impl SinkRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkRecordStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkRecordStatus) -> bool {
        match self.status {
            SinkRecordStatus::Draft => {
                match next {
                    SinkRecordStatus::Active | SinkRecordStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkRecordStatus::Pending => {
                match next {
                    SinkRecordStatus::Suspended | SinkRecordStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkRecordStatus::Active => {
                match next {
                    SinkRecordStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkRecordStatus::Suspended => {
                return next == SinkRecordStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.threshold
    }

    pub fn copy(&self) -> SinkRecord {
        SinkRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            active: self.active,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a sink record in status draft.
pub fn new_sink_record(code: &str, title: &str) -> SinkRecord {
    SinkRecord {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        active: false,
        threshold: 0.0,
        status: SinkRecordStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_sink_record("code-687", "title-599");
        assert_eq!(item.status, SinkRecordStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_sink_record("code-687", "title-599");
        if item.move_to(SinkRecordStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkRecordStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_sink_record("code-687", "title-599");
        item.status = SinkRecordStatus::Closed;
        assert!(item.move_to(SinkRecordStatus::Draft).is_err(), "expected move to fail");
    }
}
