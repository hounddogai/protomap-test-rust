use super::*;

use acme_kit::text;

/// Where a failover checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverCheckpointStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FailoverCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for FailoverCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a failover checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverCheckpoint {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub cost: f64,
    pub ttl_seconds: i64,
    pub timezone: String,
    pub capacity: i64,
    pub status: FailoverCheckpointStatus,
}

impl FailoverCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverCheckpointStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverCheckpointStatus) -> bool {
        match self.status {
            FailoverCheckpointStatus::Draft => {
                match next {
                    FailoverCheckpointStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverCheckpointStatus::Pending => {
                match next {
                    FailoverCheckpointStatus::Active | FailoverCheckpointStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverCheckpointStatus::Active => {
                return next == FailoverCheckpointStatus::Suspended;
            }
            FailoverCheckpointStatus::Suspended => {
                return next == FailoverCheckpointStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverCheckpoint {
        FailoverCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            cost: self.cost,
            ttl_seconds: self.ttl_seconds,
            timezone: self.timezone.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a failover checkpoint in status draft.
pub fn new_failover_checkpoint(code: &str, title: &str) -> FailoverCheckpoint {
    FailoverCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        segment: "".to_string(),
        cost: 0.0,
        ttl_seconds: 0,
        timezone: "".to_string(),
        capacity: 0,
        status: FailoverCheckpointStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_failover_checkpoint("code-843", "title-833");
        assert_eq!(item.status, FailoverCheckpointStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_failover_checkpoint("code-843", "title-833");
        if item.move_to(FailoverCheckpointStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverCheckpointStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_failover_checkpoint("code-843", "title-833");
        item.status = FailoverCheckpointStatus::Closed;
        assert!(
            item.move_to(FailoverCheckpointStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
