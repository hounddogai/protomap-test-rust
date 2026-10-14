use super::*;

use acme_kit::text;

/// Where a default health check preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultHealthCheckPreferenceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DefaultHealthCheckPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DefaultHealthCheckPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default health check preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultHealthCheckPreference {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub notes: String,
    pub name: String,
    pub window_days: i64,
    pub status: DefaultHealthCheckPreferenceStatus,
}

impl DefaultHealthCheckPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultHealthCheckPreferenceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultHealthCheckPreferenceStatus) -> bool {
        match self.status {
            DefaultHealthCheckPreferenceStatus::Proposed => {
                return next == DefaultHealthCheckPreferenceStatus::Rejected;
            }
            DefaultHealthCheckPreferenceStatus::Approved => {
                return next == DefaultHealthCheckPreferenceStatus::Rejected;
            }
            DefaultHealthCheckPreferenceStatus::Rejected => {
                return next == DefaultHealthCheckPreferenceStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultHealthCheckPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default health check preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.window_days as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultHealthCheckPreference {
        DefaultHealthCheckPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            notes: self.notes.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a default health check preference in status proposed.
pub fn new_default_health_check_preference(code: &str, label: &str) -> DefaultHealthCheckPreference {
    DefaultHealthCheckPreference {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        notes: "".to_string(),
        name: "".to_string(),
        window_days: 0,
        status: DefaultHealthCheckPreferenceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_default_health_check_preference("code-558", "label-567");
        assert_eq!(item.status, DefaultHealthCheckPreferenceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_default_health_check_preference("code-558", "label-567");
        if item.move_to(DefaultHealthCheckPreferenceStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultHealthCheckPreferenceStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_default_health_check_preference("code-558", "label-567");
        item.status = DefaultHealthCheckPreferenceStatus::Withdrawn;
        assert!(
            item.move_to(DefaultHealthCheckPreferenceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
