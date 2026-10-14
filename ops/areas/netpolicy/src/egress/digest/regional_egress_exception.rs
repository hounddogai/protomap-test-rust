use super::*;

use acme_kit::text;

/// Where a regional egress exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalEgressExceptionStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionalEgressExceptionStatus {
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

impl std::fmt::Display for RegionalEgressExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional egress exception, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEgressException {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub archived: bool,
    pub host: String,
    pub phone: String,
    pub status: RegionalEgressExceptionStatus,
}

impl RegionalEgressException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalEgressExceptionStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalEgressExceptionStatus) -> bool {
        match self.status {
            RegionalEgressExceptionStatus::New => {
                match next {
                    RegionalEgressExceptionStatus::Queued | RegionalEgressExceptionStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalEgressExceptionStatus::Queued => {
                return next == RegionalEgressExceptionStatus::Failed;
            }
            RegionalEgressExceptionStatus::Running => {
                return next == RegionalEgressExceptionStatus::Failed;
            }
            RegionalEgressExceptionStatus::Done => {
                match next {
                    RegionalEgressExceptionStatus::Failed => {
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

    pub fn move_to(&mut self, next: RegionalEgressExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional egress exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional egress exception", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.sequence as f64
    }

    pub fn copy(&self) -> RegionalEgressException {
        RegionalEgressException {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            archived: self.archived,
            host: self.host.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional egress exception in status new.
pub fn new_regional_egress_exception(code: &str, name: &str) -> RegionalEgressException {
    RegionalEgressException {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        archived: false,
        host: "".to_string(),
        phone: "".to_string(),
        status: RegionalEgressExceptionStatus::New,
    }
}
