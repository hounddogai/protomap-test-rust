use super::*;

use acme_kit::text;

/// Where a delegation checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationCheckpointStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DelegationCheckpointStatus {
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

impl std::fmt::Display for DelegationCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a delegation checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationCheckpoint {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub cluster: String,
    pub ttl_seconds: i64,
    pub weight: f64,
    pub tags: Vec<String>,
    pub status: DelegationCheckpointStatus,
}

impl DelegationCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationCheckpointStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationCheckpointStatus) -> bool {
        match self.status {
            DelegationCheckpointStatus::Draft => {
                match next {
                    DelegationCheckpointStatus::Active | DelegationCheckpointStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationCheckpointStatus::Pending => {
                return next == DelegationCheckpointStatus::Closed;
            }
            DelegationCheckpointStatus::Active => {
                match next {
                    DelegationCheckpointStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationCheckpointStatus::Suspended => {
                return next == DelegationCheckpointStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DelegationCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationCheckpoint {
        DelegationCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            cluster: self.cluster.clone(),
            ttl_seconds: self.ttl_seconds,
            weight: self.weight,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a delegation checkpoint in status draft.
pub fn new_delegation_checkpoint(code: &str, name: &str) -> DelegationCheckpoint {
    DelegationCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        cluster: "".to_string(),
        ttl_seconds: 0,
        weight: 0.0,
        tags: Vec::new(),
        status: DelegationCheckpointStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_delegation_checkpoint("code-572", "name-561");
        assert_eq!(item.status, DelegationCheckpointStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_delegation_checkpoint("code-572", "name-561");
        if item.move_to(DelegationCheckpointStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationCheckpointStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_delegation_checkpoint("code-572", "name-561");
        item.status = DelegationCheckpointStatus::Closed;
        assert!(
            item.move_to(DelegationCheckpointStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
