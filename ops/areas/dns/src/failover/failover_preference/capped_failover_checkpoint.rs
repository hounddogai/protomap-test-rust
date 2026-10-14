use super::*;

use acme_kit::text;

/// Where a capped failover checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedFailoverCheckpointStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CappedFailoverCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CappedFailoverCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped failover checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedFailoverCheckpoint {
    pub code: String,
    pub name: String,
    pub host: String,
    pub discount: f64,
    pub region: String,
    pub status: CappedFailoverCheckpointStatus,
}

impl CappedFailoverCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedFailoverCheckpointStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedFailoverCheckpointStatus) -> bool {
        match self.status {
            CappedFailoverCheckpointStatus::Requested => {
                return next == CappedFailoverCheckpointStatus::Granted
                    || next == CappedFailoverCheckpointStatus::Expired;
            }
            CappedFailoverCheckpointStatus::Granted => {
                match next {
                    CappedFailoverCheckpointStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedFailoverCheckpointStatus::Revoked => {
                return next == CappedFailoverCheckpointStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedFailoverCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped failover checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.discount
    }

    pub fn copy(&self) -> CappedFailoverCheckpoint {
        CappedFailoverCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            discount: self.discount,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a capped failover checkpoint in status requested.
pub fn new_capped_failover_checkpoint(code: &str, name: &str) -> CappedFailoverCheckpoint {
    CappedFailoverCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        discount: 0.0,
        region: "".to_string(),
        status: CappedFailoverCheckpointStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_capped_failover_checkpoint("code-748", "name-507");
        assert_eq!(item.status, CappedFailoverCheckpointStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_capped_failover_checkpoint("code-748", "name-507");
        if item.move_to(CappedFailoverCheckpointStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedFailoverCheckpointStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_capped_failover_checkpoint("code-748", "name-507");
        item.status = CappedFailoverCheckpointStatus::Expired;
        assert!(
            item.move_to(CappedFailoverCheckpointStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
