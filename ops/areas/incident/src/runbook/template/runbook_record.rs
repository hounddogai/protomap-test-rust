use super::*;

use acme_kit::text;

/// Where a runbook record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookRecordStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RunbookRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RunbookRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A runbook record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookRecord {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub burst: i64,
    pub margin: f64,
    pub revision: i64,
    pub ceiling: f64,
    pub unit: String,
    pub region: String,
    pub cluster: String,
    pub status: RunbookRecordStatus,
}

impl RunbookRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookRecordStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookRecordStatus) -> bool {
        match self.status {
            RunbookRecordStatus::Requested => {
                match next {
                    RunbookRecordStatus::Granted | RunbookRecordStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookRecordStatus::Granted => {
                return next == RunbookRecordStatus::Expired;
            }
            RunbookRecordStatus::Revoked => {
                return next == RunbookRecordStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RunbookRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "runbook record", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.margin * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookRecord {
        RunbookRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            burst: self.burst,
            margin: self.margin,
            revision: self.revision,
            ceiling: self.ceiling,
            unit: self.unit.clone(),
            region: self.region.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a runbook record in status requested.
pub fn new_runbook_record(code: &str, title: &str) -> RunbookRecord {
    RunbookRecord {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        burst: 0,
        margin: 0.0,
        revision: 0,
        ceiling: 0.0,
        unit: "".to_string(),
        region: "".to_string(),
        cluster: "".to_string(),
        status: RunbookRecordStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_runbook_record("code-717", "title-206");
        assert_eq!(item.status, RunbookRecordStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_runbook_record("code-717", "title-206");
        if item.move_to(RunbookRecordStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookRecordStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_runbook_record("code-717", "title-206");
        item.status = RunbookRecordStatus::Expired;
        assert!(
            item.move_to(RunbookRecordStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
