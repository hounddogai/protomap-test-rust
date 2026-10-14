use super::*;

use acme_kit::text;

/// Where a seasonal rollout preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalRolloutPreferenceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SeasonalRolloutPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for SeasonalRolloutPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal rollout preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRolloutPreference {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub label: String,
    pub margin: f64,
    pub status: SeasonalRolloutPreferenceStatus,
}

impl SeasonalRolloutPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalRolloutPreferenceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalRolloutPreferenceStatus) -> bool {
        match self.status {
            SeasonalRolloutPreferenceStatus::Planned => {
                match next {
                    SeasonalRolloutPreferenceStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalRolloutPreferenceStatus::Scheduled => {
                return next == SeasonalRolloutPreferenceStatus::InProgress
                    || next == SeasonalRolloutPreferenceStatus::Cancelled;
            }
            SeasonalRolloutPreferenceStatus::InProgress => {
                return next == SeasonalRolloutPreferenceStatus::Cancelled;
            }
            SeasonalRolloutPreferenceStatus::Completed => {
                match next {
                    SeasonalRolloutPreferenceStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: SeasonalRolloutPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal rollout preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "seasonal rollout preference", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.port as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalRolloutPreference {
        SeasonalRolloutPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            label: self.label.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a seasonal rollout preference in status planned.
pub fn new_seasonal_rollout_preference(code: &str, title: &str) -> SeasonalRolloutPreference {
    SeasonalRolloutPreference {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        label: "".to_string(),
        margin: 0.0,
        status: SeasonalRolloutPreferenceStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_seasonal_rollout_preference("code-133", "title-516");
        assert_eq!(item.status, SeasonalRolloutPreferenceStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_seasonal_rollout_preference("code-133", "title-516");
        if item.move_to(SeasonalRolloutPreferenceStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalRolloutPreferenceStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_seasonal_rollout_preference("code-133", "title-516");
        item.status = SeasonalRolloutPreferenceStatus::Cancelled;
        assert!(
            item.move_to(SeasonalRolloutPreferenceStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
