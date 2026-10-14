use super::*;

use acme_kit::text;

/// Where a lenient usage forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientUsageForecastStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientUsageForecastStatus {
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

impl std::fmt::Display for LenientUsageForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient usage forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientUsageForecast {
    pub code: String,
    pub name: String,
    pub pinned: bool,
    pub checksum: String,
    pub summary: String,
    pub discount: f64,
    pub margin: f64,
    pub country: String,
    pub category: String,
    pub cipher: String,
    pub burst: i64,
    pub status: LenientUsageForecastStatus,
}

impl LenientUsageForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientUsageForecastStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientUsageForecastStatus) -> bool {
        match self.status {
            LenientUsageForecastStatus::Open => {
                return next == LenientUsageForecastStatus::Resolved || next == LenientUsageForecastStatus::Reopened;
            }
            LenientUsageForecastStatus::InReview => {
                return next == LenientUsageForecastStatus::Reopened;
            }
            LenientUsageForecastStatus::Resolved => {
                match next {
                    LenientUsageForecastStatus::Reopened | LenientUsageForecastStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientUsageForecastStatus::Reopened => {
                match next {
                    LenientUsageForecastStatus::Archived => {
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

    pub fn move_to(&mut self, next: LenientUsageForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient usage forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        (self.margin * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientUsageForecast {
        LenientUsageForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            pinned: self.pinned,
            checksum: self.checksum.clone(),
            summary: self.summary.clone(),
            discount: self.discount,
            margin: self.margin,
            country: self.country.clone(),
            category: self.category.clone(),
            cipher: self.cipher.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a lenient usage forecast in status open.
pub fn new_lenient_usage_forecast(code: &str, name: &str) -> LenientUsageForecast {
    LenientUsageForecast {
        code: code.to_string(),
        name: name.to_string(),
        pinned: false,
        checksum: "".to_string(),
        summary: "".to_string(),
        discount: 0.0,
        margin: 0.0,
        country: "".to_string(),
        category: "".to_string(),
        cipher: "".to_string(),
        burst: 0,
        status: LenientUsageForecastStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_lenient_usage_forecast("code-316", "name-581");
        assert_eq!(item.status, LenientUsageForecastStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_lenient_usage_forecast("code-316", "name-581");
        if item.move_to(LenientUsageForecastStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientUsageForecastStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_lenient_usage_forecast("code-316", "name-581");
        item.status = LenientUsageForecastStatus::Archived;
        assert!(
            item.move_to(LenientUsageForecastStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
