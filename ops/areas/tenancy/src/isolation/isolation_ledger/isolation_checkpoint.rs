use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a isolation checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationCheckpointStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl IsolationCheckpointStatus {
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

impl std::fmt::Display for IsolationCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A isolation checkpoint. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationCheckpoint {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub segment: String,
    pub key_id: String,
    pub region: String,
    pub created_at: i64,
    pub attributes: BTreeMap<String, String>,
    pub ratio: f64,
    pub visible: bool,
    pub status: IsolationCheckpointStatus,
}

impl IsolationCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationCheckpointStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationCheckpointStatus) -> bool {
        match self.status {
            IsolationCheckpointStatus::Draft => {
                return next == IsolationCheckpointStatus::Active;
            }
            IsolationCheckpointStatus::Pending => {
                return next == IsolationCheckpointStatus::Suspended || next == IsolationCheckpointStatus::Closed;
            }
            IsolationCheckpointStatus::Active => {
                match next {
                    IsolationCheckpointStatus::Suspended | IsolationCheckpointStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationCheckpointStatus::Suspended => {
                return next == IsolationCheckpointStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IsolationCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn net_load(&self) -> f64 {
        ((self.created_at as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationCheckpoint {
        IsolationCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            segment: self.segment.clone(),
            key_id: self.key_id.clone(),
            region: self.region.clone(),
            created_at: self.created_at,
            attributes: self.attributes.clone(),
            ratio: self.ratio,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a isolation checkpoint in status draft.
pub fn new_isolation_checkpoint(code: &str, name: &str) -> IsolationCheckpoint {
    IsolationCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        segment: "".to_string(),
        key_id: "".to_string(),
        region: "".to_string(),
        created_at: 0,
        attributes: BTreeMap::new(),
        ratio: 0.0,
        visible: false,
        status: IsolationCheckpointStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_isolation_checkpoint("code-538", "name-280");
        assert_eq!(item.status, IsolationCheckpointStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_isolation_checkpoint("code-538", "name-280");
        if item.move_to(IsolationCheckpointStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationCheckpointStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_isolation_checkpoint("code-538", "name-280");
        item.status = IsolationCheckpointStatus::Closed;
        assert!(
            item.move_to(IsolationCheckpointStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
