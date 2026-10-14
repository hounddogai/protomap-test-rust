use super::*;

use acme_kit::text;

/// Where a adaptive page variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptivePageVarianceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptivePageVarianceStatus {
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

impl std::fmt::Display for AdaptivePageVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive page variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePageVariance {
    pub code: String,
    pub title: String,
    pub unit: String,
    pub discount: f64,
    pub summary: String,
    pub revision: i64,
    pub status: AdaptivePageVarianceStatus,
}

impl AdaptivePageVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptivePageVarianceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptivePageVarianceStatus) -> bool {
        match self.status {
            AdaptivePageVarianceStatus::Open => {
                return next == AdaptivePageVarianceStatus::Reopened;
            }
            AdaptivePageVarianceStatus::InReview => {
                return next == AdaptivePageVarianceStatus::Reopened;
            }
            AdaptivePageVarianceStatus::Resolved => {
                match next {
                    AdaptivePageVarianceStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptivePageVarianceStatus::Reopened => {
                return next == AdaptivePageVarianceStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptivePageVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive page variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.revision as f64 - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptivePageVariance {
        AdaptivePageVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            unit: self.unit.clone(),
            discount: self.discount,
            summary: self.summary.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a adaptive page variance in status open.
pub fn new_adaptive_page_variance(code: &str, title: &str) -> AdaptivePageVariance {
    AdaptivePageVariance {
        code: code.to_string(),
        title: title.to_string(),
        unit: "".to_string(),
        discount: 0.0,
        summary: "".to_string(),
        revision: 0,
        status: AdaptivePageVarianceStatus::Open,
    }
}
