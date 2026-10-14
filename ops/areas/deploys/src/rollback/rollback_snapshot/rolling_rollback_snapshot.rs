use super::*;

use acme_kit::text;

/// Where a rolling rollback snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingRollbackSnapshotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingRollbackSnapshotStatus {
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

impl std::fmt::Display for RollingRollbackSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling rollback snapshot as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRollbackSnapshot {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub capacity: i64,
    pub weight: f64,
    pub minimum: f64,
    pub region: String,
    pub version: i64,
    pub burst: i64,
    pub starts_at: i64,
    pub port: i64,
    pub status: RollingRollbackSnapshotStatus,
}

impl RollingRollbackSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingRollbackSnapshotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingRollbackSnapshotStatus) -> bool {
        match self.status {
            RollingRollbackSnapshotStatus::Draft => {
                return next == RollingRollbackSnapshotStatus::Pending
                    || next == RollingRollbackSnapshotStatus::Suspended;
            }
            RollingRollbackSnapshotStatus::Pending => {
                match next {
                    RollingRollbackSnapshotStatus::Active | RollingRollbackSnapshotStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRollbackSnapshotStatus::Active => {
                return next == RollingRollbackSnapshotStatus::Suspended
                    || next == RollingRollbackSnapshotStatus::Closed;
            }
            RollingRollbackSnapshotStatus::Suspended => {
                match next {
                    RollingRollbackSnapshotStatus::Closed => {
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

    pub fn move_to(&mut self, next: RollingRollbackSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling rollback snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.minimum - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingRollbackSnapshot {
        RollingRollbackSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            capacity: self.capacity,
            weight: self.weight,
            minimum: self.minimum,
            region: self.region.clone(),
            version: self.version,
            burst: self.burst,
            starts_at: self.starts_at,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a rolling rollback snapshot in status draft.
pub fn new_rolling_rollback_snapshot(code: &str, name: &str) -> RollingRollbackSnapshot {
    RollingRollbackSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        capacity: 0,
        weight: 0.0,
        minimum: 0.0,
        region: "".to_string(),
        version: 0,
        burst: 0,
        starts_at: 0,
        port: 0,
        status: RollingRollbackSnapshotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rolling_rollback_snapshot("code-435", "name-880");
        assert_eq!(item.status, RollingRollbackSnapshotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_rolling_rollback_snapshot("code-435", "name-880");
        if item.move_to(RollingRollbackSnapshotStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingRollbackSnapshotStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rolling_rollback_snapshot("code-435", "name-880");
        item.status = RollingRollbackSnapshotStatus::Closed;
        assert!(
            item.move_to(RollingRollbackSnapshotStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
