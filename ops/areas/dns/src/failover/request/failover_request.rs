use super::*;

use acme_kit::text;

/// Where a failover request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverRequestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FailoverRequestStatus {
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

impl std::fmt::Display for FailoverRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a failover request and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRequest {
    pub code: String,
    pub label: String,
    pub active: bool,
    pub reviewer: String,
    pub retries: i64,
    pub country: String,
    pub burst: i64,
    pub currency: String,
    pub quota_units: i64,
    pub status: FailoverRequestStatus,
}

impl FailoverRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverRequestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverRequestStatus) -> bool {
        match self.status {
            FailoverRequestStatus::New => {
                return next == FailoverRequestStatus::Done;
            }
            FailoverRequestStatus::Queued => {
                match next {
                    FailoverRequestStatus::Running | FailoverRequestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverRequestStatus::Running => {
                return next == FailoverRequestStatus::Done || next == FailoverRequestStatus::Failed;
            }
            FailoverRequestStatus::Done => {
                return next == FailoverRequestStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.retries as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverRequest {
        FailoverRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            active: self.active,
            reviewer: self.reviewer.clone(),
            retries: self.retries,
            country: self.country.clone(),
            burst: self.burst,
            currency: self.currency.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a failover request in status new.
pub fn new_failover_request(code: &str, label: &str) -> FailoverRequest {
    FailoverRequest {
        code: code.to_string(),
        label: label.to_string(),
        active: false,
        reviewer: "".to_string(),
        retries: 0,
        country: "".to_string(),
        burst: 0,
        currency: "".to_string(),
        quota_units: 0,
        status: FailoverRequestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_failover_request("code-66", "label-559");
        assert_eq!(item.status, FailoverRequestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_failover_request("code-66", "label-559");
        if item.move_to(FailoverRequestStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverRequestStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_failover_request("code-66", "label-559");
        item.status = FailoverRequestStatus::Failed;
        assert!(
            item.move_to(FailoverRequestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
