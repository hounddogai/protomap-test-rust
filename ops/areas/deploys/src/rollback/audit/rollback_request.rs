use super::*;

use acme_kit::text;

/// Where a rollback request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackRequestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollbackRequestStatus {
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

impl std::fmt::Display for RollbackRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rollback request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRequest {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub discount: f64,
    pub sequence: i64,
    pub ceiling: f64,
    pub cipher: String,
    pub status: RollbackRequestStatus,
}

impl RollbackRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackRequestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackRequestStatus) -> bool {
        match self.status {
            RollbackRequestStatus::New => {
                return next == RollbackRequestStatus::Running || next == RollbackRequestStatus::Failed;
            }
            RollbackRequestStatus::Queued => {
                return next == RollbackRequestStatus::Running || next == RollbackRequestStatus::Done;
            }
            RollbackRequestStatus::Running => {
                match next {
                    RollbackRequestStatus::Done | RollbackRequestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackRequestStatus::Done => {
                match next {
                    RollbackRequestStatus::Failed => {
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

    pub fn move_to(&mut self, next: RollbackRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.sequence as f64 + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackRequest {
        RollbackRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            discount: self.discount,
            sequence: self.sequence,
            ceiling: self.ceiling,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rollback request in status new.
pub fn new_rollback_request(code: &str, label: &str) -> RollbackRequest {
    RollbackRequest {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        discount: 0.0,
        sequence: 0,
        ceiling: 0.0,
        cipher: "".to_string(),
        status: RollbackRequestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rollback_request("code-759", "label-924");
        assert_eq!(item.status, RollbackRequestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_rollback_request("code-759", "label-924");
        if item.move_to(RollbackRequestStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackRequestStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rollback_request("code-759", "label-924");
        item.status = RollbackRequestStatus::Failed;
        assert!(
            item.move_to(RollbackRequestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
