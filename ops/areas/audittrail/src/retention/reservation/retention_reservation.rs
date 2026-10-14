use super::*;

use acme_kit::text;

/// Where a retention reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionReservationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RetentionReservationStatus {
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

impl std::fmt::Display for RetentionReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A retention reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionReservation {
    pub code: String,
    pub title: String,
    pub category: String,
    pub port: i64,
    pub retries: i64,
    pub locale: String,
    pub scope: String,
    pub host: String,
    pub status: RetentionReservationStatus,
}

impl RetentionReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionReservationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionReservationStatus) -> bool {
        match self.status {
            RetentionReservationStatus::New => {
                return next == RetentionReservationStatus::Failed;
            }
            RetentionReservationStatus::Queued => {
                match next {
                    RetentionReservationStatus::Running | RetentionReservationStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionReservationStatus::Running => {
                return next == RetentionReservationStatus::Failed;
            }
            RetentionReservationStatus::Done => {
                return next == RetentionReservationStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.retries as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionReservation {
        RetentionReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            port: self.port,
            retries: self.retries,
            locale: self.locale.clone(),
            scope: self.scope.clone(),
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a retention reservation in status new.
pub fn new_retention_reservation(code: &str, title: &str) -> RetentionReservation {
    RetentionReservation {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        port: 0,
        retries: 0,
        locale: "".to_string(),
        scope: "".to_string(),
        host: "".to_string(),
        status: RetentionReservationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_retention_reservation("code-311", "title-288");
        assert_eq!(item.status, RetentionReservationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_retention_reservation("code-311", "title-288");
        if item.move_to(RetentionReservationStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionReservationStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_retention_reservation("code-311", "title-288");
        item.status = RetentionReservationStatus::Failed;
        assert!(
            item.move_to(RetentionReservationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
