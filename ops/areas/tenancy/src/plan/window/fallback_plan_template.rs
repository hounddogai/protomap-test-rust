use super::*;

use acme_kit::text;

/// Where a fallback plan template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackPlanTemplateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FallbackPlanTemplateStatus {
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

impl std::fmt::Display for FallbackPlanTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback plan template, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanTemplate {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub attempts: i64,
    pub rotation_days: i64,
    pub status: FallbackPlanTemplateStatus,
}

impl FallbackPlanTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackPlanTemplateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackPlanTemplateStatus) -> bool {
        match self.status {
            FallbackPlanTemplateStatus::Idle => {
                return next == FallbackPlanTemplateStatus::Ready;
            }
            FallbackPlanTemplateStatus::Warming => {
                return next == FallbackPlanTemplateStatus::Ready || next == FallbackPlanTemplateStatus::Stopped;
            }
            FallbackPlanTemplateStatus::Ready => {
                return next == FallbackPlanTemplateStatus::Draining || next == FallbackPlanTemplateStatus::Stopped;
            }
            FallbackPlanTemplateStatus::Draining => {
                return next == FallbackPlanTemplateStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackPlanTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback plan template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback plan template", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackPlanTemplate {
        FallbackPlanTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            attempts: self.attempts,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a fallback plan template in status idle.
pub fn new_fallback_plan_template(code: &str, label: &str) -> FallbackPlanTemplate {
    FallbackPlanTemplate {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        attempts: 0,
        rotation_days: 0,
        status: FallbackPlanTemplateStatus::Idle,
    }
}
