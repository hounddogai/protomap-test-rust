use super::*;

use acme_kit::text;

/// Where a forecast audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastAuditStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ForecastAuditStatus {
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

impl std::fmt::Display for ForecastAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One forecast audit, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAudit {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub score: f64,
    pub label: String,
    pub tenant_id: String,
    pub unit: String,
    pub segment: String,
    pub title: String,
    pub status: ForecastAuditStatus,
}

impl ForecastAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastAuditStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastAuditStatus) -> bool {
        match self.status {
            ForecastAuditStatus::New => {
                return next == ForecastAuditStatus::Done || next == ForecastAuditStatus::Failed;
            }
            ForecastAuditStatus::Queued => {
                match next {
                    ForecastAuditStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastAuditStatus::Running => {
                match next {
                    ForecastAuditStatus::Done | ForecastAuditStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastAuditStatus::Done => {
                match next {
                    ForecastAuditStatus::Failed => {
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

    pub fn move_to(&mut self, next: ForecastAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> ForecastAudit {
        ForecastAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            score: self.score,
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            unit: self.unit.clone(),
            segment: self.segment.clone(),
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a forecast audit in status new.
pub fn new_forecast_audit(code: &str, name: &str) -> ForecastAudit {
    ForecastAudit {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        score: 0.0,
        label: "".to_string(),
        tenant_id: "".to_string(),
        unit: "".to_string(),
        segment: "".to_string(),
        title: "".to_string(),
        status: ForecastAuditStatus::New,
    }
}
