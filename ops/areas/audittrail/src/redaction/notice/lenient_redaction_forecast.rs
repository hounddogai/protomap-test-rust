use super::*;

use acme_kit::text;

/// Where a lenient redaction forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientRedactionForecastStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientRedactionForecastStatus {
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

impl std::fmt::Display for LenientRedactionForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient redaction forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRedactionForecast {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub batch_size: i64,
    pub version: i64,
    pub status: LenientRedactionForecastStatus,
}

impl LenientRedactionForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientRedactionForecastStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientRedactionForecastStatus) -> bool {
        match self.status {
            LenientRedactionForecastStatus::Open => {
                match next {
                    LenientRedactionForecastStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRedactionForecastStatus::InReview => {
                return next == LenientRedactionForecastStatus::Resolved;
            }
            LenientRedactionForecastStatus::Resolved => {
                match next {
                    LenientRedactionForecastStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRedactionForecastStatus::Reopened => {
                match next {
                    LenientRedactionForecastStatus::Archived => {
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

    pub fn move_to(&mut self, next: LenientRedactionForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient redaction forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.version as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientRedactionForecast {
        LenientRedactionForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            batch_size: self.batch_size,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a lenient redaction forecast in status open.
pub fn new_lenient_redaction_forecast(code: &str, label: &str) -> LenientRedactionForecast {
    LenientRedactionForecast {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        batch_size: 0,
        version: 0,
        status: LenientRedactionForecastStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_lenient_redaction_forecast("code-12", "label-678");
        assert_eq!(item.status, LenientRedactionForecastStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_lenient_redaction_forecast("code-12", "label-678");
        if item.move_to(LenientRedactionForecastStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientRedactionForecastStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_lenient_redaction_forecast("code-12", "label-678");
        item.status = LenientRedactionForecastStatus::Archived;
        assert!(
            item.move_to(LenientRedactionForecastStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
