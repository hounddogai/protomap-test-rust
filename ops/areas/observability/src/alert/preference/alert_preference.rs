use super::*;

use acme_kit::text;

/// Where a alert preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertPreferenceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AlertPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AlertPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A alert preference as the alert team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertPreference {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub threshold: f64,
    pub port: i64,
    pub active: bool,
    pub locale: String,
    pub window_days: i64,
    pub score: f64,
    pub status: AlertPreferenceStatus,
}

impl AlertPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertPreferenceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertPreferenceStatus) -> bool {
        match self.status {
            AlertPreferenceStatus::Created => {
                return next == AlertPreferenceStatus::Retired;
            }
            AlertPreferenceStatus::Validated => {
                return next == AlertPreferenceStatus::Published;
            }
            AlertPreferenceStatus::Published => {
                return next == AlertPreferenceStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AlertPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.threshold + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertPreference {
        AlertPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            threshold: self.threshold,
            port: self.port,
            active: self.active,
            locale: self.locale.clone(),
            window_days: self.window_days,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a alert preference in status created.
pub fn new_alert_preference(code: &str, title: &str) -> AlertPreference {
    AlertPreference {
        code: code.to_string(),
        title: title.to_string(),
        segment: "".to_string(),
        threshold: 0.0,
        port: 0,
        active: false,
        locale: "".to_string(),
        window_days: 0,
        score: 0.0,
        status: AlertPreferenceStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_alert_preference("code-237", "title-748");
        assert_eq!(item.status, AlertPreferenceStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_alert_preference("code-237", "title-748");
        if item.move_to(AlertPreferenceStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertPreferenceStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_alert_preference("code-237", "title-748");
        item.status = AlertPreferenceStatus::Retired;
        assert!(
            item.move_to(AlertPreferenceStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
