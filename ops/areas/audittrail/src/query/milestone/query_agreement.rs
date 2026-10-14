use super::*;

use acme_kit::text;

/// Where a query agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryAgreementStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl QueryAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for QueryAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a query agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryAgreement {
    pub code: String,
    pub title: String,
    pub duration_minutes: i64,
    pub updated_at: i64,
    pub principal: String,
    pub minimum: f64,
    pub quota_units: i64,
    pub rotation_days: i64,
    pub status: QueryAgreementStatus,
}

impl QueryAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryAgreementStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryAgreementStatus) -> bool {
        match self.status {
            QueryAgreementStatus::Proposed => {
                return next == QueryAgreementStatus::Withdrawn;
            }
            QueryAgreementStatus::Approved => {
                return next == QueryAgreementStatus::Rejected;
            }
            QueryAgreementStatus::Rejected => {
                return next == QueryAgreementStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query agreement {} cannot move from {} to {}",
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
        format!("{} {}", "query agreement", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.rotation_days as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryAgreement {
        QueryAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            updated_at: self.updated_at,
            principal: self.principal.clone(),
            minimum: self.minimum,
            quota_units: self.quota_units,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a query agreement in status proposed.
pub fn new_query_agreement(code: &str, title: &str) -> QueryAgreement {
    QueryAgreement {
        code: code.to_string(),
        title: title.to_string(),
        duration_minutes: 0,
        updated_at: 0,
        principal: "".to_string(),
        minimum: 0.0,
        quota_units: 0,
        rotation_days: 0,
        status: QueryAgreementStatus::Proposed,
    }
}
