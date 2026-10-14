use super::*;

use acme_kit::text;

/// Where a tiered runbook template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredRunbookTemplateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredRunbookTemplateStatus {
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

impl std::fmt::Display for TieredRunbookTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered runbook template as the runbook team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookTemplate {
    pub code: String,
    pub title: String,
    pub quota_units: i64,
    pub score: f64,
    pub timezone: String,
    pub fingerprint: String,
    pub capacity: i64,
    pub window_days: i64,
    pub phone: String,
    pub unit: String,
    pub status: TieredRunbookTemplateStatus,
}

impl TieredRunbookTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredRunbookTemplateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredRunbookTemplateStatus) -> bool {
        match self.status {
            TieredRunbookTemplateStatus::Idle => {
                match next {
                    TieredRunbookTemplateStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRunbookTemplateStatus::Warming => {
                match next {
                    TieredRunbookTemplateStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRunbookTemplateStatus::Ready => {
                return next == TieredRunbookTemplateStatus::Draining;
            }
            TieredRunbookTemplateStatus::Draining => {
                return next == TieredRunbookTemplateStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredRunbookTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered runbook template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.window_days as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredRunbookTemplate {
        TieredRunbookTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            score: self.score,
            timezone: self.timezone.clone(),
            fingerprint: self.fingerprint.clone(),
            capacity: self.capacity,
            window_days: self.window_days,
            phone: self.phone.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a tiered runbook template in status idle.
pub fn new_tiered_runbook_template(code: &str, title: &str) -> TieredRunbookTemplate {
    TieredRunbookTemplate {
        code: code.to_string(),
        title: title.to_string(),
        quota_units: 0,
        score: 0.0,
        timezone: "".to_string(),
        fingerprint: "".to_string(),
        capacity: 0,
        window_days: 0,
        phone: "".to_string(),
        unit: "".to_string(),
        status: TieredRunbookTemplateStatus::Idle,
    }
}
