use super::*;

use acme_kit::text;

/// Where a pending plan template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingPlanTemplateStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PendingPlanTemplateStatus {
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

impl std::fmt::Display for PendingPlanTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending plan template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanTemplate {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub currency: String,
    pub rotation_days: i64,
    pub weight: f64,
    pub created_at: i64,
    pub floor_value: f64,
    pub fingerprint: String,
    pub status: PendingPlanTemplateStatus,
}

impl PendingPlanTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingPlanTemplateStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingPlanTemplateStatus) -> bool {
        match self.status {
            PendingPlanTemplateStatus::New => {
                return next == PendingPlanTemplateStatus::Running || next == PendingPlanTemplateStatus::Done;
            }
            PendingPlanTemplateStatus::Queued => {
                match next {
                    PendingPlanTemplateStatus::Running | PendingPlanTemplateStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPlanTemplateStatus::Running => {
                match next {
                    PendingPlanTemplateStatus::Done | PendingPlanTemplateStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPlanTemplateStatus::Done => {
                match next {
                    PendingPlanTemplateStatus::Failed => {
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

    pub fn move_to(&mut self, next: PendingPlanTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending plan template {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.created_at as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingPlanTemplate {
        PendingPlanTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            currency: self.currency.clone(),
            rotation_days: self.rotation_days,
            weight: self.weight,
            created_at: self.created_at,
            floor_value: self.floor_value,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending plan template in status new.
pub fn new_pending_plan_template(code: &str, name: &str) -> PendingPlanTemplate {
    PendingPlanTemplate {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        currency: "".to_string(),
        rotation_days: 0,
        weight: 0.0,
        created_at: 0,
        floor_value: 0.0,
        fingerprint: "".to_string(),
        status: PendingPlanTemplateStatus::New,
    }
}
