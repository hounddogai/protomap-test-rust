use super::*;

use acme_kit::text;

/// Where a batched sink exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedSinkExceptionStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedSinkExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedSinkExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched sink exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSinkException {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub attempts: i64,
    pub region: String,
    pub floor_value: f64,
    pub audience: String,
    pub grace_days: i64,
    pub rate: f64,
    pub host: String,
    pub maximum: f64,
    pub status: BatchedSinkExceptionStatus,
}

impl BatchedSinkException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedSinkExceptionStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedSinkExceptionStatus) -> bool {
        match self.status {
            BatchedSinkExceptionStatus::Requested => {
                match next {
                    BatchedSinkExceptionStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedSinkExceptionStatus::Granted => {
                return next == BatchedSinkExceptionStatus::Revoked || next == BatchedSinkExceptionStatus::Expired;
            }
            BatchedSinkExceptionStatus::Revoked => {
                return next == BatchedSinkExceptionStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedSinkExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched sink exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.maximum + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedSinkException {
        BatchedSinkException {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            attempts: self.attempts,
            region: self.region.clone(),
            floor_value: self.floor_value,
            audience: self.audience.clone(),
            grace_days: self.grace_days,
            rate: self.rate,
            host: self.host.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a batched sink exception in status requested.
pub fn new_batched_sink_exception(code: &str, title: &str) -> BatchedSinkException {
    BatchedSinkException {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        attempts: 0,
        region: "".to_string(),
        floor_value: 0.0,
        audience: "".to_string(),
        grace_days: 0,
        rate: 0.0,
        host: "".to_string(),
        maximum: 0.0,
        status: BatchedSinkExceptionStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_batched_sink_exception("code-697", "title-169");
        assert_eq!(item.status, BatchedSinkExceptionStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_batched_sink_exception("code-697", "title-169");
        if item.move_to(BatchedSinkExceptionStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedSinkExceptionStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_batched_sink_exception("code-697", "title-169");
        item.status = BatchedSinkExceptionStatus::Expired;
        assert!(
            item.move_to(BatchedSinkExceptionStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
