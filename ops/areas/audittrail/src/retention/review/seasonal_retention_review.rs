use super::*;

use acme_kit::text;

/// Where a seasonal retention review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalRetentionReviewStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalRetentionReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalRetentionReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal retention review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRetentionReview {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub expires_at: i64,
    pub external_ref: String,
    pub principal: String,
    pub issuer: String,
    pub approved: bool,
    pub tags: Vec<String>,
    pub quota_units: i64,
    pub currency: String,
    pub status: SeasonalRetentionReviewStatus,
}

impl SeasonalRetentionReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalRetentionReviewStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalRetentionReviewStatus) -> bool {
        match self.status {
            SeasonalRetentionReviewStatus::Requested => {
                return next == SeasonalRetentionReviewStatus::Revoked
                    || next == SeasonalRetentionReviewStatus::Expired;
            }
            SeasonalRetentionReviewStatus::Granted => {
                match next {
                    SeasonalRetentionReviewStatus::Revoked | SeasonalRetentionReviewStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalRetentionReviewStatus::Revoked => {
                return next == SeasonalRetentionReviewStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalRetentionReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal retention review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "seasonal retention review", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.quota_units as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalRetentionReview {
        SeasonalRetentionReview {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            expires_at: self.expires_at,
            external_ref: self.external_ref.clone(),
            principal: self.principal.clone(),
            issuer: self.issuer.clone(),
            approved: self.approved,
            tags: self.tags.clone(),
            quota_units: self.quota_units,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a seasonal retention review in status requested.
pub fn new_seasonal_retention_review(code: &str, label: &str) -> SeasonalRetentionReview {
    SeasonalRetentionReview {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        expires_at: 0,
        external_ref: "".to_string(),
        principal: "".to_string(),
        issuer: "".to_string(),
        approved: false,
        tags: Vec::new(),
        quota_units: 0,
        currency: "".to_string(),
        status: SeasonalRetentionReviewStatus::Requested,
    }
}
