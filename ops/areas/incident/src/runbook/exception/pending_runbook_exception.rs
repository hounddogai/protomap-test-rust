use super::*;

use acme_kit::text;

/// Where a pending runbook exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRunbookExceptionStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PendingRunbookExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PendingRunbookExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending runbook exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookException {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub capacity: i64,
    pub window_days: i64,
    pub minimum: f64,
    pub key_id: String,
    pub phone: String,
    pub rotation_days: i64,
    pub title: String,
    pub weight: f64,
    pub status: PendingRunbookExceptionStatus,
}

impl PendingRunbookException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRunbookExceptionStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRunbookExceptionStatus) -> bool {
        match self.status {
            PendingRunbookExceptionStatus::Requested => {
                return next == PendingRunbookExceptionStatus::Revoked;
            }
            PendingRunbookExceptionStatus::Granted => {
                match next {
                    PendingRunbookExceptionStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRunbookExceptionStatus::Revoked => {
                return next == PendingRunbookExceptionStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingRunbookExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending runbook exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.weight * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRunbookException {
        PendingRunbookException {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            capacity: self.capacity,
            window_days: self.window_days,
            minimum: self.minimum,
            key_id: self.key_id.clone(),
            phone: self.phone.clone(),
            rotation_days: self.rotation_days,
            title: self.title.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending runbook exception in status requested.
pub fn new_pending_runbook_exception(code: &str, name: &str) -> PendingRunbookException {
    PendingRunbookException {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        capacity: 0,
        window_days: 0,
        minimum: 0.0,
        key_id: "".to_string(),
        phone: "".to_string(),
        rotation_days: 0,
        title: "".to_string(),
        weight: 0.0,
        status: PendingRunbookExceptionStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_pending_runbook_exception("code-949", "name-961");
        assert_eq!(item.status, PendingRunbookExceptionStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_pending_runbook_exception("code-949", "name-961");
        if item.move_to(PendingRunbookExceptionStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingRunbookExceptionStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_pending_runbook_exception("code-949", "name-961");
        item.status = PendingRunbookExceptionStatus::Expired;
        assert!(
            item.move_to(PendingRunbookExceptionStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
