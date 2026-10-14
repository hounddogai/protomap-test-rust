use super::*;

use acme_kit::text;

/// Where a default federation forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultFederationForecastStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultFederationForecastStatus {
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

impl std::fmt::Display for DefaultFederationForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default federation forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFederationForecast {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub external_ref: String,
    pub quantity: i64,
    pub visible: bool,
    pub pinned: bool,
    pub status: DefaultFederationForecastStatus,
}

impl DefaultFederationForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultFederationForecastStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultFederationForecastStatus) -> bool {
        match self.status {
            DefaultFederationForecastStatus::Open => {
                match next {
                    DefaultFederationForecastStatus::Resolved | DefaultFederationForecastStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultFederationForecastStatus::InReview => {
                return next == DefaultFederationForecastStatus::Resolved
                    || next == DefaultFederationForecastStatus::Archived;
            }
            DefaultFederationForecastStatus::Resolved => {
                return next == DefaultFederationForecastStatus::Reopened
                    || next == DefaultFederationForecastStatus::Archived;
            }
            DefaultFederationForecastStatus::Reopened => {
                return next == DefaultFederationForecastStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultFederationForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default federation forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "default federation forecast", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> DefaultFederationForecast {
        DefaultFederationForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            external_ref: self.external_ref.clone(),
            quantity: self.quantity,
            visible: self.visible,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a default federation forecast in status open.
pub fn new_default_federation_forecast(code: &str, label: &str) -> DefaultFederationForecast {
    DefaultFederationForecast {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        external_ref: "".to_string(),
        quantity: 0,
        visible: false,
        pinned: false,
        status: DefaultFederationForecastStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_default_federation_forecast("code-765", "label-941");
        assert_eq!(item.status, DefaultFederationForecastStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_default_federation_forecast("code-765", "label-941");
        if item.move_to(DefaultFederationForecastStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultFederationForecastStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_default_federation_forecast("code-765", "label-941");
        item.status = DefaultFederationForecastStatus::Archived;
        assert!(
            item.move_to(DefaultFederationForecastStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
