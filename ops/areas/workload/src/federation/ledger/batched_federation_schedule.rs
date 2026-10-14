use super::*;

use acme_kit::text;

/// Where a batched federation schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedFederationScheduleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedFederationScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedFederationScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched federation schedule as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationSchedule {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub issuer: String,
    pub title: String,
    pub approved: bool,
    pub ttl_seconds: i64,
    pub ceiling: f64,
    pub revision: i64,
    pub status: BatchedFederationScheduleStatus,
}

impl BatchedFederationSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedFederationScheduleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedFederationScheduleStatus) -> bool {
        match self.status {
            BatchedFederationScheduleStatus::Requested => {
                return next == BatchedFederationScheduleStatus::Granted;
            }
            BatchedFederationScheduleStatus::Granted => {
                return next == BatchedFederationScheduleStatus::Revoked
                    || next == BatchedFederationScheduleStatus::Expired;
            }
            BatchedFederationScheduleStatus::Revoked => {
                return next == BatchedFederationScheduleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedFederationScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched federation schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched federation schedule", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedFederationSchedule {
        BatchedFederationSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            issuer: self.issuer.clone(),
            title: self.title.clone(),
            approved: self.approved,
            ttl_seconds: self.ttl_seconds,
            ceiling: self.ceiling,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched federation schedule in status requested.
pub fn new_batched_federation_schedule(code: &str, name: &str) -> BatchedFederationSchedule {
    BatchedFederationSchedule {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        issuer: "".to_string(),
        title: "".to_string(),
        approved: false,
        ttl_seconds: 0,
        ceiling: 0.0,
        revision: 0,
        status: BatchedFederationScheduleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_batched_federation_schedule("code-196", "name-844");
        assert_eq!(item.status, BatchedFederationScheduleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_batched_federation_schedule("code-196", "name-844");
        if item.move_to(BatchedFederationScheduleStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedFederationScheduleStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_batched_federation_schedule("code-196", "name-844");
        item.status = BatchedFederationScheduleStatus::Expired;
        assert!(
            item.move_to(BatchedFederationScheduleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
