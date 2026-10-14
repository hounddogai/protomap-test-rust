use super::*;

use acme_kit::text;

/// Where a action item baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemBaselineStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ActionItemBaselineStatus {
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

impl std::fmt::Display for ActionItemBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a action item baseline and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemBaseline {
    pub code: String,
    pub label: String,
    pub quota_units: i64,
    pub timezone: String,
    pub expires_at: i64,
    pub grace_days: i64,
    pub rate: f64,
    pub region: String,
    pub tags: Vec<String>,
    pub weight: f64,
    pub status: ActionItemBaselineStatus,
}

impl ActionItemBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemBaselineStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemBaselineStatus) -> bool {
        match self.status {
            ActionItemBaselineStatus::Planned => {
                return next == ActionItemBaselineStatus::Scheduled;
            }
            ActionItemBaselineStatus::Scheduled => {
                match next {
                    ActionItemBaselineStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemBaselineStatus::InProgress => {
                return next == ActionItemBaselineStatus::Cancelled;
            }
            ActionItemBaselineStatus::Completed => {
                return next == ActionItemBaselineStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.quota_units as f64) * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemBaseline {
        ActionItemBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            timezone: self.timezone.clone(),
            expires_at: self.expires_at,
            grace_days: self.grace_days,
            rate: self.rate,
            region: self.region.clone(),
            tags: self.tags.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a action item baseline in status planned.
pub fn new_action_item_baseline(code: &str, label: &str) -> ActionItemBaseline {
    ActionItemBaseline {
        code: code.to_string(),
        label: label.to_string(),
        quota_units: 0,
        timezone: "".to_string(),
        expires_at: 0,
        grace_days: 0,
        rate: 0.0,
        region: "".to_string(),
        tags: Vec::new(),
        weight: 0.0,
        status: ActionItemBaselineStatus::Planned,
    }
}
