use super::*;

use acme_kit::text;

/// Where a global federation forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalFederationForecastStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GlobalFederationForecastStatus {
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

impl std::fmt::Display for GlobalFederationForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global federation forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFederationForecast {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub reviewer: String,
    pub locale: String,
    pub attempts: i64,
    pub cipher: String,
    pub active: bool,
    pub status: GlobalFederationForecastStatus,
}

impl GlobalFederationForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalFederationForecastStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalFederationForecastStatus) -> bool {
        match self.status {
            GlobalFederationForecastStatus::Open => {
                return next == GlobalFederationForecastStatus::InReview
                    || next == GlobalFederationForecastStatus::Resolved;
            }
            GlobalFederationForecastStatus::InReview => {
                match next {
                    GlobalFederationForecastStatus::Resolved | GlobalFederationForecastStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalFederationForecastStatus::Resolved => {
                match next {
                    GlobalFederationForecastStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalFederationForecastStatus::Reopened => {
                return next == GlobalFederationForecastStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalFederationForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global federation forecast {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.updated_at as f64 - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalFederationForecast {
        GlobalFederationForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            reviewer: self.reviewer.clone(),
            locale: self.locale.clone(),
            attempts: self.attempts,
            cipher: self.cipher.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a global federation forecast in status open.
pub fn new_global_federation_forecast(code: &str, name: &str) -> GlobalFederationForecast {
    GlobalFederationForecast {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        reviewer: "".to_string(),
        locale: "".to_string(),
        attempts: 0,
        cipher: "".to_string(),
        active: false,
        status: GlobalFederationForecastStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_global_federation_forecast("code-556", "name-584");
        assert_eq!(item.status, GlobalFederationForecastStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_global_federation_forecast("code-556", "name-584");
        if item.move_to(GlobalFederationForecastStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalFederationForecastStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_global_federation_forecast("code-556", "name-584");
        item.status = GlobalFederationForecastStatus::Archived;
        assert!(
            item.move_to(GlobalFederationForecastStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
