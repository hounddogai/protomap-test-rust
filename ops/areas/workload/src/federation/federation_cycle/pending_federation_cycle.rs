use super::*;

use acme_kit::text;

/// Where a pending federation cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingFederationCycleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PendingFederationCycleStatus {
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

impl std::fmt::Display for PendingFederationCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending federation cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationCycle {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub cost: f64,
    pub retries: i64,
    pub external_ref: String,
    pub approved: bool,
    pub ratio: f64,
    pub reviewer: String,
    pub status: PendingFederationCycleStatus,
}

impl PendingFederationCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingFederationCycleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingFederationCycleStatus) -> bool {
        match self.status {
            PendingFederationCycleStatus::New => {
                match next {
                    PendingFederationCycleStatus::Running | PendingFederationCycleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingFederationCycleStatus::Queued => {
                match next {
                    PendingFederationCycleStatus::Running | PendingFederationCycleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingFederationCycleStatus::Running => {
                match next {
                    PendingFederationCycleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingFederationCycleStatus::Done => {
                match next {
                    PendingFederationCycleStatus::Failed => {
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

    pub fn move_to(&mut self, next: PendingFederationCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending federation cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ratio - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingFederationCycle {
        PendingFederationCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            key_id: self.key_id.clone(),
            cost: self.cost,
            retries: self.retries,
            external_ref: self.external_ref.clone(),
            approved: self.approved,
            ratio: self.ratio,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending federation cycle in status new.
pub fn new_pending_federation_cycle(code: &str, label: &str) -> PendingFederationCycle {
    PendingFederationCycle {
        code: code.to_string(),
        label: label.to_string(),
        key_id: "".to_string(),
        cost: 0.0,
        retries: 0,
        external_ref: "".to_string(),
        approved: false,
        ratio: 0.0,
        reviewer: "".to_string(),
        status: PendingFederationCycleStatus::New,
    }
}
