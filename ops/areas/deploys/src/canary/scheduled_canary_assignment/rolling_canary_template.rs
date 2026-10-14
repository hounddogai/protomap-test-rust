use super::*;

use acme_kit::text;

/// Where a rolling canary template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingCanaryTemplateStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingCanaryTemplateStatus {
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

impl std::fmt::Display for RollingCanaryTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling canary template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingCanaryTemplate {
    pub code: String,
    pub name: String,
    pub region: String,
    pub external_ref: String,
    pub issuer: String,
    pub maximum: f64,
    pub phone: String,
    pub capacity: i64,
    pub status: RollingCanaryTemplateStatus,
}

impl RollingCanaryTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingCanaryTemplateStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingCanaryTemplateStatus) -> bool {
        match self.status {
            RollingCanaryTemplateStatus::Draft => {
                return next == RollingCanaryTemplateStatus::Suspended;
            }
            RollingCanaryTemplateStatus::Pending => {
                return next == RollingCanaryTemplateStatus::Suspended;
            }
            RollingCanaryTemplateStatus::Active => {
                return next == RollingCanaryTemplateStatus::Closed;
            }
            RollingCanaryTemplateStatus::Suspended => {
                return next == RollingCanaryTemplateStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingCanaryTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling canary template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.capacity as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingCanaryTemplate {
        RollingCanaryTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            external_ref: self.external_ref.clone(),
            issuer: self.issuer.clone(),
            maximum: self.maximum,
            phone: self.phone.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a rolling canary template in status draft.
pub fn new_rolling_canary_template(code: &str, name: &str) -> RollingCanaryTemplate {
    RollingCanaryTemplate {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        external_ref: "".to_string(),
        issuer: "".to_string(),
        maximum: 0.0,
        phone: "".to_string(),
        capacity: 0,
        status: RollingCanaryTemplateStatus::Draft,
    }
}
