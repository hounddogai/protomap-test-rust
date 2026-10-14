use super::*;

use acme_kit::text;

/// Where a rolling flow log checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingFlowLogCheckpointStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingFlowLogCheckpointStatus {
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

impl std::fmt::Display for RollingFlowLogCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling flow log checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingFlowLogCheckpoint {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub version: i64,
    pub amount: f64,
    pub rate: f64,
    pub tags: Vec<String>,
    pub cluster: String,
    pub tenant_id: String,
    pub status: RollingFlowLogCheckpointStatus,
}

impl RollingFlowLogCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingFlowLogCheckpointStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingFlowLogCheckpointStatus) -> bool {
        match self.status {
            RollingFlowLogCheckpointStatus::Draft => {
                match next {
                    RollingFlowLogCheckpointStatus::Active | RollingFlowLogCheckpointStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingFlowLogCheckpointStatus::Pending => {
                match next {
                    RollingFlowLogCheckpointStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingFlowLogCheckpointStatus::Active => {
                match next {
                    RollingFlowLogCheckpointStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingFlowLogCheckpointStatus::Suspended => {
                return next == RollingFlowLogCheckpointStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingFlowLogCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling flow log checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_value(&self) -> f64 {
        ((self.ceiling - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingFlowLogCheckpoint {
        RollingFlowLogCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            version: self.version,
            amount: self.amount,
            rate: self.rate,
            tags: self.tags.clone(),
            cluster: self.cluster.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a rolling flow log checkpoint in status draft.
pub fn new_rolling_flow_log_checkpoint(code: &str, name: &str) -> RollingFlowLogCheckpoint {
    RollingFlowLogCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        version: 0,
        amount: 0.0,
        rate: 0.0,
        tags: Vec::new(),
        cluster: "".to_string(),
        tenant_id: "".to_string(),
        status: RollingFlowLogCheckpointStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rolling_flow_log_checkpoint("code-968", "name-665");
        assert_eq!(item.status, RollingFlowLogCheckpointStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_rolling_flow_log_checkpoint("code-968", "name-665");
        if item.move_to(RollingFlowLogCheckpointStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingFlowLogCheckpointStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rolling_flow_log_checkpoint("code-968", "name-665");
        item.status = RollingFlowLogCheckpointStatus::Closed;
        assert!(
            item.move_to(RollingFlowLogCheckpointStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
