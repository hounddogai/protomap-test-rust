use super::*;

use acme_kit::text;

/// Where a adaptive suspension checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveSuspensionCheckpointStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AdaptiveSuspensionCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for AdaptiveSuspensionCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive suspension checkpoint as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSuspensionCheckpoint {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub reviewer: String,
    pub issuer: String,
    pub external_ref: String,
    pub approved: bool,
    pub contact_email: String,
    pub ratio: f64,
    pub notes: String,
    pub status: AdaptiveSuspensionCheckpointStatus,
}

impl AdaptiveSuspensionCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveSuspensionCheckpointStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveSuspensionCheckpointStatus) -> bool {
        match self.status {
            AdaptiveSuspensionCheckpointStatus::New => {
                match next {
                    AdaptiveSuspensionCheckpointStatus::Running | AdaptiveSuspensionCheckpointStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveSuspensionCheckpointStatus::Queued => {
                return next == AdaptiveSuspensionCheckpointStatus::Running
                    || next == AdaptiveSuspensionCheckpointStatus::Done;
            }
            AdaptiveSuspensionCheckpointStatus::Running => {
                match next {
                    AdaptiveSuspensionCheckpointStatus::Done | AdaptiveSuspensionCheckpointStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveSuspensionCheckpointStatus::Done => {
                return next == AdaptiveSuspensionCheckpointStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveSuspensionCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive suspension checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.amount + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveSuspensionCheckpoint {
        AdaptiveSuspensionCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            reviewer: self.reviewer.clone(),
            issuer: self.issuer.clone(),
            external_ref: self.external_ref.clone(),
            approved: self.approved,
            contact_email: self.contact_email.clone(),
            ratio: self.ratio,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a adaptive suspension checkpoint in status new.
pub fn new_adaptive_suspension_checkpoint(code: &str, name: &str) -> AdaptiveSuspensionCheckpoint {
    AdaptiveSuspensionCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        reviewer: "".to_string(),
        issuer: "".to_string(),
        external_ref: "".to_string(),
        approved: false,
        contact_email: "".to_string(),
        ratio: 0.0,
        notes: "".to_string(),
        status: AdaptiveSuspensionCheckpointStatus::New,
    }
}
