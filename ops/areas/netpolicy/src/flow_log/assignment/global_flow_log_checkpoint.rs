use super::*;

use acme_kit::text;

/// Where a global flow log checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalFlowLogCheckpointStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GlobalFlowLogCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GlobalFlowLogCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global flow log checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFlowLogCheckpoint {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub pinned: bool,
    pub floor_value: f64,
    pub batch_size: i64,
    pub key_id: String,
    pub weight: f64,
    pub timezone: String,
    pub status: GlobalFlowLogCheckpointStatus,
}

impl GlobalFlowLogCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalFlowLogCheckpointStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalFlowLogCheckpointStatus) -> bool {
        match self.status {
            GlobalFlowLogCheckpointStatus::Requested => {
                match next {
                    GlobalFlowLogCheckpointStatus::Granted | GlobalFlowLogCheckpointStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalFlowLogCheckpointStatus::Granted => {
                match next {
                    GlobalFlowLogCheckpointStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalFlowLogCheckpointStatus::Revoked => {
                match next {
                    GlobalFlowLogCheckpointStatus::Expired => {
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

    pub fn move_to(&mut self, next: GlobalFlowLogCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global flow log checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "global flow log checkpoint", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.rotation_days as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalFlowLogCheckpoint {
        GlobalFlowLogCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            pinned: self.pinned,
            floor_value: self.floor_value,
            batch_size: self.batch_size,
            key_id: self.key_id.clone(),
            weight: self.weight,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global flow log checkpoint in status requested.
pub fn new_global_flow_log_checkpoint(code: &str, label: &str) -> GlobalFlowLogCheckpoint {
    GlobalFlowLogCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        pinned: false,
        floor_value: 0.0,
        batch_size: 0,
        key_id: "".to_string(),
        weight: 0.0,
        timezone: "".to_string(),
        status: GlobalFlowLogCheckpointStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_global_flow_log_checkpoint("code-795", "label-248");
        assert_eq!(item.status, GlobalFlowLogCheckpointStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_global_flow_log_checkpoint("code-795", "label-248");
        if item.move_to(GlobalFlowLogCheckpointStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalFlowLogCheckpointStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_global_flow_log_checkpoint("code-795", "label-248");
        item.status = GlobalFlowLogCheckpointStatus::Expired;
        assert!(
            item.move_to(GlobalFlowLogCheckpointStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
