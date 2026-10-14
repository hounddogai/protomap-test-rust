use super::*;

use acme_kit::text;

/// Where a seasonal timeline agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalTimelineAgreementStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SeasonalTimelineAgreementStatus {
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

impl std::fmt::Display for SeasonalTimelineAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal timeline agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTimelineAgreement {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub cost: f64,
    pub archived: bool,
    pub updated_at: i64,
    pub phone: String,
    pub owner_id: String,
    pub quota_units: i64,
    pub rotation_days: i64,
    pub status: SeasonalTimelineAgreementStatus,
}

impl SeasonalTimelineAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalTimelineAgreementStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalTimelineAgreementStatus) -> bool {
        match self.status {
            SeasonalTimelineAgreementStatus::New => {
                return next == SeasonalTimelineAgreementStatus::Queued;
            }
            SeasonalTimelineAgreementStatus::Queued => {
                return next == SeasonalTimelineAgreementStatus::Running;
            }
            SeasonalTimelineAgreementStatus::Running => {
                return next == SeasonalTimelineAgreementStatus::Done;
            }
            SeasonalTimelineAgreementStatus::Done => {
                return next == SeasonalTimelineAgreementStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalTimelineAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal timeline agreement {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.updated_at as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalTimelineAgreement {
        SeasonalTimelineAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            cost: self.cost,
            archived: self.archived,
            updated_at: self.updated_at,
            phone: self.phone.clone(),
            owner_id: self.owner_id.clone(),
            quota_units: self.quota_units,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a seasonal timeline agreement in status new.
pub fn new_seasonal_timeline_agreement(code: &str, label: &str) -> SeasonalTimelineAgreement {
    SeasonalTimelineAgreement {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        cost: 0.0,
        archived: false,
        updated_at: 0,
        phone: "".to_string(),
        owner_id: "".to_string(),
        quota_units: 0,
        rotation_days: 0,
        status: SeasonalTimelineAgreementStatus::New,
    }
}
