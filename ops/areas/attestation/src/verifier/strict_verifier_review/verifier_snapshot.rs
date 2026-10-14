use super::*;

use acme_kit::text;

/// Where a verifier snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierSnapshotStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl VerifierSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for VerifierSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A verifier snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSnapshot {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub threshold: f64,
    pub owner_id: String,
    pub cluster: String,
    pub ttl_seconds: i64,
    pub description: String,
    pub status: VerifierSnapshotStatus,
}

impl VerifierSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierSnapshotStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierSnapshotStatus) -> bool {
        match self.status {
            VerifierSnapshotStatus::Created => {
                match next {
                    VerifierSnapshotStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierSnapshotStatus::Validated => {
                match next {
                    VerifierSnapshotStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierSnapshotStatus::Published => {
                match next {
                    VerifierSnapshotStatus::Retired => {
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

    pub fn move_to(&mut self, next: VerifierSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "verifier snapshot", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierSnapshot {
        VerifierSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            threshold: self.threshold,
            owner_id: self.owner_id.clone(),
            cluster: self.cluster.clone(),
            ttl_seconds: self.ttl_seconds,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a verifier snapshot in status created.
pub fn new_verifier_snapshot(code: &str, name: &str) -> VerifierSnapshot {
    VerifierSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        threshold: 0.0,
        owner_id: "".to_string(),
        cluster: "".to_string(),
        ttl_seconds: 0,
        description: "".to_string(),
        status: VerifierSnapshotStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_verifier_snapshot("code-934", "name-953");
        assert_eq!(item.status, VerifierSnapshotStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_verifier_snapshot("code-934", "name-953");
        if item.move_to(VerifierSnapshotStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierSnapshotStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_verifier_snapshot("code-934", "name-953");
        item.status = VerifierSnapshotStatus::Retired;
        assert!(
            item.move_to(VerifierSnapshotStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
