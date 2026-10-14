use super::*;

use acme_kit::text;

/// Where a default identity forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultIdentityForecastStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultIdentityForecastStatus {
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

impl std::fmt::Display for DefaultIdentityForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default identity forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIdentityForecast {
    pub code: String,
    pub label: String,
    pub category: String,
    pub duration_minutes: i64,
    pub visible: bool,
    pub burst: i64,
    pub phone: String,
    pub amount: f64,
    pub ttl_seconds: i64,
    pub notes: String,
    pub threshold: f64,
    pub status: DefaultIdentityForecastStatus,
}

impl DefaultIdentityForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultIdentityForecastStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultIdentityForecastStatus) -> bool {
        match self.status {
            DefaultIdentityForecastStatus::Open => {
                return next == DefaultIdentityForecastStatus::Reopened;
            }
            DefaultIdentityForecastStatus::InReview => {
                match next {
                    DefaultIdentityForecastStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultIdentityForecastStatus::Resolved => {
                return next == DefaultIdentityForecastStatus::Reopened;
            }
            DefaultIdentityForecastStatus::Reopened => {
                return next == DefaultIdentityForecastStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultIdentityForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default identity forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "default identity forecast", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.burst as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultIdentityForecast {
        DefaultIdentityForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            duration_minutes: self.duration_minutes,
            visible: self.visible,
            burst: self.burst,
            phone: self.phone.clone(),
            amount: self.amount,
            ttl_seconds: self.ttl_seconds,
            notes: self.notes.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a default identity forecast in status open.
pub fn new_default_identity_forecast(code: &str, label: &str) -> DefaultIdentityForecast {
    DefaultIdentityForecast {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        duration_minutes: 0,
        visible: false,
        burst: 0,
        phone: "".to_string(),
        amount: 0.0,
        ttl_seconds: 0,
        notes: "".to_string(),
        threshold: 0.0,
        status: DefaultIdentityForecastStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_default_identity_forecast("code-980", "label-872");
        assert_eq!(item.status, DefaultIdentityForecastStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_default_identity_forecast("code-980", "label-872");
        if item.move_to(DefaultIdentityForecastStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultIdentityForecastStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_default_identity_forecast("code-980", "label-872");
        item.status = DefaultIdentityForecastStatus::Archived;
        assert!(
            item.move_to(DefaultIdentityForecastStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
