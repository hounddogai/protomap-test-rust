use super::*;

use acme_kit::text;

/// Where a pending evidence forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingEvidenceForecastStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PendingEvidenceForecastStatus {
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

impl std::fmt::Display for PendingEvidenceForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending evidence forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingEvidenceForecast {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub country: String,
    pub segment: String,
    pub principal: String,
    pub status: PendingEvidenceForecastStatus,
}

impl PendingEvidenceForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingEvidenceForecastStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingEvidenceForecastStatus) -> bool {
        match self.status {
            PendingEvidenceForecastStatus::Open => {
                return next == PendingEvidenceForecastStatus::InReview;
            }
            PendingEvidenceForecastStatus::InReview => {
                match next {
                    PendingEvidenceForecastStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingEvidenceForecastStatus::Resolved => {
                match next {
                    PendingEvidenceForecastStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingEvidenceForecastStatus::Reopened => {
                return next == PendingEvidenceForecastStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingEvidenceForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending evidence forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> PendingEvidenceForecast {
        PendingEvidenceForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            country: self.country.clone(),
            segment: self.segment.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a pending evidence forecast in status open.
pub fn new_pending_evidence_forecast(code: &str, title: &str) -> PendingEvidenceForecast {
    PendingEvidenceForecast {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        country: "".to_string(),
        segment: "".to_string(),
        principal: "".to_string(),
        status: PendingEvidenceForecastStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_pending_evidence_forecast("code-924", "title-445");
        assert_eq!(item.status, PendingEvidenceForecastStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_pending_evidence_forecast("code-924", "title-445");
        if item.move_to(PendingEvidenceForecastStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingEvidenceForecastStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_pending_evidence_forecast("code-924", "title-445");
        item.status = PendingEvidenceForecastStatus::Archived;
        assert!(
            item.move_to(PendingEvidenceForecastStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
