use super::*;

use acme_kit::text;

/// Where a rolling plan checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingPlanCheckpointStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollingPlanCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollingPlanCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling plan checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanCheckpoint {
    pub code: String,
    pub title: String,
    pub visible: bool,
    pub budget: f64,
    pub cluster: String,
    pub cipher: String,
    pub tenant_id: String,
    pub segment: String,
    pub batch_size: i64,
    pub status: RollingPlanCheckpointStatus,
}

impl RollingPlanCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingPlanCheckpointStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingPlanCheckpointStatus) -> bool {
        match self.status {
            RollingPlanCheckpointStatus::Requested => {
                match next {
                    RollingPlanCheckpointStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingPlanCheckpointStatus::Granted => {
                return next == RollingPlanCheckpointStatus::Revoked;
            }
            RollingPlanCheckpointStatus::Revoked => {
                return next == RollingPlanCheckpointStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingPlanCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling plan checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.batch_size as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingPlanCheckpoint {
        RollingPlanCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            visible: self.visible,
            budget: self.budget,
            cluster: self.cluster.clone(),
            cipher: self.cipher.clone(),
            tenant_id: self.tenant_id.clone(),
            segment: self.segment.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a rolling plan checkpoint in status requested.
pub fn new_rolling_plan_checkpoint(code: &str, title: &str) -> RollingPlanCheckpoint {
    RollingPlanCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        visible: false,
        budget: 0.0,
        cluster: "".to_string(),
        cipher: "".to_string(),
        tenant_id: "".to_string(),
        segment: "".to_string(),
        batch_size: 0,
        status: RollingPlanCheckpointStatus::Requested,
    }
}
