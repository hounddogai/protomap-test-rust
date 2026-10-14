use super::*;

use acme_kit::text;

/// Where a grant record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantRecordStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GrantRecordStatus {
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

impl std::fmt::Display for GrantRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One grant record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantRecord {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub description: String,
    pub burst: i64,
    pub rate: f64,
    pub approved: bool,
    pub duration_minutes: i64,
    pub owner_id: String,
    pub status: GrantRecordStatus,
}

impl GrantRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantRecordStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantRecordStatus) -> bool {
        match self.status {
            GrantRecordStatus::New => {
                return next == GrantRecordStatus::Queued || next == GrantRecordStatus::Running;
            }
            GrantRecordStatus::Queued => {
                match next {
                    GrantRecordStatus::Running | GrantRecordStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantRecordStatus::Running => {
                match next {
                    GrantRecordStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantRecordStatus::Done => {
                match next {
                    GrantRecordStatus::Failed => {
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

    pub fn move_to(&mut self, next: GrantRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantRecord {
        GrantRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            description: self.description.clone(),
            burst: self.burst,
            rate: self.rate,
            approved: self.approved,
            duration_minutes: self.duration_minutes,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a grant record in status new.
pub fn new_grant_record(code: &str, label: &str) -> GrantRecord {
    GrantRecord {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        description: "".to_string(),
        burst: 0,
        rate: 0.0,
        approved: false,
        duration_minutes: 0,
        owner_id: "".to_string(),
        status: GrantRecordStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_grant_record("code-800", "label-765");
        assert_eq!(item.status, GrantRecordStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_grant_record("code-800", "label-765");
        if item.move_to(GrantRecordStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantRecordStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_grant_record("code-800", "label-765");
        item.status = GrantRecordStatus::Failed;
        assert!(item.move_to(GrantRecordStatus::New).is_err(), "expected move to fail");
    }
}
