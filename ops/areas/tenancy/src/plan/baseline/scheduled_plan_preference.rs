use super::*;

use acme_kit::text;

/// Where a scheduled plan preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledPlanPreferenceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ScheduledPlanPreferenceStatus {
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

impl std::fmt::Display for ScheduledPlanPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled plan preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPlanPreference {
    pub code: String,
    pub name: String,
    pub ttl_seconds: i64,
    pub quantity: i64,
    pub title: String,
    pub limit: i64,
    pub category: String,
    pub tags: Vec<String>,
    pub retries: i64,
    pub floor_value: f64,
    pub status: ScheduledPlanPreferenceStatus,
}

impl ScheduledPlanPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledPlanPreferenceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledPlanPreferenceStatus) -> bool {
        match self.status {
            ScheduledPlanPreferenceStatus::Planned => {
                match next {
                    ScheduledPlanPreferenceStatus::Scheduled | ScheduledPlanPreferenceStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledPlanPreferenceStatus::Scheduled => {
                return next == ScheduledPlanPreferenceStatus::InProgress
                    || next == ScheduledPlanPreferenceStatus::Cancelled;
            }
            ScheduledPlanPreferenceStatus::InProgress => {
                match next {
                    ScheduledPlanPreferenceStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledPlanPreferenceStatus::Completed => {
                match next {
                    ScheduledPlanPreferenceStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ScheduledPlanPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled plan preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.retries as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledPlanPreference {
        ScheduledPlanPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            quantity: self.quantity,
            title: self.title.clone(),
            limit: self.limit,
            category: self.category.clone(),
            tags: self.tags.clone(),
            retries: self.retries,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a scheduled plan preference in status planned.
pub fn new_scheduled_plan_preference(code: &str, name: &str) -> ScheduledPlanPreference {
    ScheduledPlanPreference {
        code: code.to_string(),
        name: name.to_string(),
        ttl_seconds: 0,
        quantity: 0,
        title: "".to_string(),
        limit: 0,
        category: "".to_string(),
        tags: Vec::new(),
        retries: 0,
        floor_value: 0.0,
        status: ScheduledPlanPreferenceStatus::Planned,
    }
}
