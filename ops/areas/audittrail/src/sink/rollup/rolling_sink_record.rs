use super::*;

use acme_kit::text;

/// Where a rolling sink record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingSinkRecordStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingSinkRecordStatus {
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

impl std::fmt::Display for RollingSinkRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling sink record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSinkRecord {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub sequence: i64,
    pub description: String,
    pub version: i64,
    pub title: String,
    pub visible: bool,
    pub approved: bool,
    pub status: RollingSinkRecordStatus,
}

impl RollingSinkRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingSinkRecordStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingSinkRecordStatus) -> bool {
        match self.status {
            RollingSinkRecordStatus::New => {
                return next == RollingSinkRecordStatus::Done;
            }
            RollingSinkRecordStatus::Queued => {
                return next == RollingSinkRecordStatus::Running || next == RollingSinkRecordStatus::Failed;
            }
            RollingSinkRecordStatus::Running => {
                match next {
                    RollingSinkRecordStatus::Done | RollingSinkRecordStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingSinkRecordStatus::Done => {
                match next {
                    RollingSinkRecordStatus::Failed => {
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

    pub fn move_to(&mut self, next: RollingSinkRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling sink record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.version as f64 + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingSinkRecord {
        RollingSinkRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            sequence: self.sequence,
            description: self.description.clone(),
            version: self.version,
            title: self.title.clone(),
            visible: self.visible,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling sink record in status new.
pub fn new_rolling_sink_record(code: &str, name: &str) -> RollingSinkRecord {
    RollingSinkRecord {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        sequence: 0,
        description: "".to_string(),
        version: 0,
        title: "".to_string(),
        visible: false,
        approved: false,
        status: RollingSinkRecordStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_sink_record("code-629", "name-335");
        assert_eq!(item.status, RollingSinkRecordStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_rolling_sink_record("code-629", "name-335");
        if item.move_to(RollingSinkRecordStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingSinkRecordStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_sink_record("code-629", "name-335");
        item.status = RollingSinkRecordStatus::Failed;
        assert!(
            item.move_to(RollingSinkRecordStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
