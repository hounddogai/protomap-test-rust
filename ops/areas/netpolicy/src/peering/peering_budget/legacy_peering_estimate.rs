use super::*;

use acme_kit::text;

/// Where a legacy peering estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyPeeringEstimateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyPeeringEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for LegacyPeeringEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy peering estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPeeringEstimate {
    pub code: String,
    pub title: String,
    pub cost: f64,
    pub ttl_seconds: i64,
    pub country: String,
    pub phone: String,
    pub summary: String,
    pub namespace: String,
    pub margin: f64,
    pub status: LegacyPeeringEstimateStatus,
}

impl LegacyPeeringEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyPeeringEstimateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyPeeringEstimateStatus) -> bool {
        match self.status {
            LegacyPeeringEstimateStatus::Open => {
                return next == LegacyPeeringEstimateStatus::InReview || next == LegacyPeeringEstimateStatus::Reopened;
            }
            LegacyPeeringEstimateStatus::InReview => {
                return next == LegacyPeeringEstimateStatus::Reopened;
            }
            LegacyPeeringEstimateStatus::Resolved => {
                return next == LegacyPeeringEstimateStatus::Reopened;
            }
            LegacyPeeringEstimateStatus::Reopened => {
                return next == LegacyPeeringEstimateStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyPeeringEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy peering estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyPeeringEstimate {
        LegacyPeeringEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            cost: self.cost,
            ttl_seconds: self.ttl_seconds,
            country: self.country.clone(),
            phone: self.phone.clone(),
            summary: self.summary.clone(),
            namespace: self.namespace.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a legacy peering estimate in status open.
pub fn new_legacy_peering_estimate(code: &str, title: &str) -> LegacyPeeringEstimate {
    LegacyPeeringEstimate {
        code: code.to_string(),
        title: title.to_string(),
        cost: 0.0,
        ttl_seconds: 0,
        country: "".to_string(),
        phone: "".to_string(),
        summary: "".to_string(),
        namespace: "".to_string(),
        margin: 0.0,
        status: LegacyPeeringEstimateStatus::Open,
    }
}
