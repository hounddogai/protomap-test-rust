use super::*;

use acme_kit::text;

/// Where a primary plan preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryPlanPreferenceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PrimaryPlanPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for PrimaryPlanPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary plan preference as the plan team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanPreference {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub updated_at: i64,
    pub description: String,
    pub status: PrimaryPlanPreferenceStatus,
}

impl PrimaryPlanPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryPlanPreferenceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryPlanPreferenceStatus) -> bool {
        match self.status {
            PrimaryPlanPreferenceStatus::Idle => {
                match next {
                    PrimaryPlanPreferenceStatus::Ready | PrimaryPlanPreferenceStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryPlanPreferenceStatus::Warming => {
                return next == PrimaryPlanPreferenceStatus::Ready || next == PrimaryPlanPreferenceStatus::Stopped;
            }
            PrimaryPlanPreferenceStatus::Ready => {
                return next == PrimaryPlanPreferenceStatus::Draining || next == PrimaryPlanPreferenceStatus::Stopped;
            }
            PrimaryPlanPreferenceStatus::Draining => {
                return next == PrimaryPlanPreferenceStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryPlanPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary plan preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary plan preference", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.updated_at as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryPlanPreference {
        PrimaryPlanPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            updated_at: self.updated_at,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a primary plan preference in status idle.
pub fn new_primary_plan_preference(code: &str, name: &str) -> PrimaryPlanPreference {
    PrimaryPlanPreference {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        updated_at: 0,
        description: "".to_string(),
        status: PrimaryPlanPreferenceStatus::Idle,
    }
}
