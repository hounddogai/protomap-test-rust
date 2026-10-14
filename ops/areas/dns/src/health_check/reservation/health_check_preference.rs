use super::*;

use acme_kit::text;

/// Where a health check preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckPreferenceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl HealthCheckPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for HealthCheckPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One health check preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckPreference {
    pub code: String,
    pub label: String,
    pub host: String,
    pub archived: bool,
    pub margin: f64,
    pub grace_days: i64,
    pub locale: String,
    pub batch_size: i64,
    pub approved: bool,
    pub status: HealthCheckPreferenceStatus,
}

impl HealthCheckPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckPreferenceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckPreferenceStatus) -> bool {
        match self.status {
            HealthCheckPreferenceStatus::New => {
                return next == HealthCheckPreferenceStatus::Queued;
            }
            HealthCheckPreferenceStatus::Queued => {
                match next {
                    HealthCheckPreferenceStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckPreferenceStatus::Running => {
                return next == HealthCheckPreferenceStatus::Done || next == HealthCheckPreferenceStatus::Failed;
            }
            HealthCheckPreferenceStatus::Done => {
                match next {
                    HealthCheckPreferenceStatus::Failed => {
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

    pub fn move_to(&mut self, next: HealthCheckPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.grace_days as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckPreference {
        HealthCheckPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            host: self.host.clone(),
            archived: self.archived,
            margin: self.margin,
            grace_days: self.grace_days,
            locale: self.locale.clone(),
            batch_size: self.batch_size,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a health check preference in status new.
pub fn new_health_check_preference(code: &str, label: &str) -> HealthCheckPreference {
    HealthCheckPreference {
        code: code.to_string(),
        label: label.to_string(),
        host: "".to_string(),
        archived: false,
        margin: 0.0,
        grace_days: 0,
        locale: "".to_string(),
        batch_size: 0,
        approved: false,
        status: HealthCheckPreferenceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_health_check_preference("code-950", "label-455");
        assert_eq!(item.status, HealthCheckPreferenceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_health_check_preference("code-950", "label-455");
        if item.move_to(HealthCheckPreferenceStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckPreferenceStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_health_check_preference("code-950", "label-455");
        item.status = HealthCheckPreferenceStatus::Failed;
        assert!(
            item.move_to(HealthCheckPreferenceStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
