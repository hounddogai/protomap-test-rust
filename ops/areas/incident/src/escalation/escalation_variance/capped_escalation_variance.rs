use super::*;

use acme_kit::text;

/// Where a capped escalation variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedEscalationVarianceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CappedEscalationVarianceStatus {
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

impl std::fmt::Display for CappedEscalationVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped escalation variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedEscalationVariance {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub contact_email: String,
    pub host: String,
    pub duration_minutes: i64,
    pub status: CappedEscalationVarianceStatus,
}

impl CappedEscalationVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedEscalationVarianceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedEscalationVarianceStatus) -> bool {
        match self.status {
            CappedEscalationVarianceStatus::New => {
                match next {
                    CappedEscalationVarianceStatus::Running | CappedEscalationVarianceStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedEscalationVarianceStatus::Queued => {
                return next == CappedEscalationVarianceStatus::Running
                    || next == CappedEscalationVarianceStatus::Failed;
            }
            CappedEscalationVarianceStatus::Running => {
                match next {
                    CappedEscalationVarianceStatus::Done | CappedEscalationVarianceStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedEscalationVarianceStatus::Done => {
                match next {
                    CappedEscalationVarianceStatus::Failed => {
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

    pub fn move_to(&mut self, next: CappedEscalationVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped escalation variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped escalation variance", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.duration_minutes as f64
    }

    pub fn copy(&self) -> CappedEscalationVariance {
        CappedEscalationVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            contact_email: self.contact_email.clone(),
            host: self.host.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a capped escalation variance in status new.
pub fn new_capped_escalation_variance(code: &str, name: &str) -> CappedEscalationVariance {
    CappedEscalationVariance {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        contact_email: "".to_string(),
        host: "".to_string(),
        duration_minutes: 0,
        status: CappedEscalationVarianceStatus::New,
    }
}
