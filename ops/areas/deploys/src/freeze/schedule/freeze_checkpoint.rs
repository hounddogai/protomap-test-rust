use super::*;

use acme_kit::text;

/// Where a freeze checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeCheckpointStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FreezeCheckpointStatus {
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

impl std::fmt::Display for FreezeCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a freeze checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeCheckpoint {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub audience: String,
    pub version: i64,
    pub active: bool,
    pub issuer: String,
    pub minimum: f64,
    pub capacity: i64,
    pub region: String,
    pub batch_size: i64,
    pub status: FreezeCheckpointStatus,
}

impl FreezeCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeCheckpointStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeCheckpointStatus) -> bool {
        match self.status {
            FreezeCheckpointStatus::Draft => {
                return next == FreezeCheckpointStatus::Suspended;
            }
            FreezeCheckpointStatus::Pending => {
                match next {
                    FreezeCheckpointStatus::Active | FreezeCheckpointStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeCheckpointStatus::Active => {
                return next == FreezeCheckpointStatus::Suspended || next == FreezeCheckpointStatus::Closed;
            }
            FreezeCheckpointStatus::Suspended => {
                match next {
                    FreezeCheckpointStatus::Closed => {
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

    pub fn move_to(&mut self, next: FreezeCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.minimum + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeCheckpoint {
        FreezeCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            audience: self.audience.clone(),
            version: self.version,
            active: self.active,
            issuer: self.issuer.clone(),
            minimum: self.minimum,
            capacity: self.capacity,
            region: self.region.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a freeze checkpoint in status draft.
pub fn new_freeze_checkpoint(code: &str, title: &str) -> FreezeCheckpoint {
    FreezeCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        audience: "".to_string(),
        version: 0,
        active: false,
        issuer: "".to_string(),
        minimum: 0.0,
        capacity: 0,
        region: "".to_string(),
        batch_size: 0,
        status: FreezeCheckpointStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_freeze_checkpoint("code-735", "title-124");
        assert_eq!(item.status, FreezeCheckpointStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_freeze_checkpoint("code-735", "title-124");
        if item.move_to(FreezeCheckpointStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeCheckpointStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_freeze_checkpoint("code-735", "title-124");
        item.status = FreezeCheckpointStatus::Closed;
        assert!(
            item.move_to(FreezeCheckpointStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
