use super::*;

use acme_kit::text;

/// Where a limit estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitEstimateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LimitEstimateStatus {
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

impl std::fmt::Display for LimitEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A limit estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitEstimate {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub active: bool,
    pub discount: f64,
    pub contact_email: String,
    pub maximum: f64,
    pub updated_at: i64,
    pub status: LimitEstimateStatus,
}

impl LimitEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitEstimateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitEstimateStatus) -> bool {
        match self.status {
            LimitEstimateStatus::Idle => {
                return next == LimitEstimateStatus::Ready;
            }
            LimitEstimateStatus::Warming => {
                return next == LimitEstimateStatus::Draining || next == LimitEstimateStatus::Stopped;
            }
            LimitEstimateStatus::Ready => {
                match next {
                    LimitEstimateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitEstimateStatus::Draining => {
                return next == LimitEstimateStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit estimate {} cannot move from {} to {}",
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
        format!("{} {}", "limit estimate", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        (self.maximum * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitEstimate {
        LimitEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            active: self.active,
            discount: self.discount,
            contact_email: self.contact_email.clone(),
            maximum: self.maximum,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a limit estimate in status idle.
pub fn new_limit_estimate(code: &str, name: &str) -> LimitEstimate {
    LimitEstimate {
        code: code.to_string(),
        name: name.to_string(),
        currency: "".to_string(),
        active: false,
        discount: 0.0,
        contact_email: "".to_string(),
        maximum: 0.0,
        updated_at: 0,
        status: LimitEstimateStatus::Idle,
    }
}
