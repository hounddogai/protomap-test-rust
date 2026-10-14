use super::*;

use acme_kit::text;

/// Where a global limit preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalLimitPreferenceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl GlobalLimitPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for GlobalLimitPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global limit preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitPreference {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub archived: bool,
    pub rotation_days: i64,
    pub duration_minutes: i64,
    pub window_days: i64,
    pub discount: f64,
    pub capacity: i64,
    pub rate: f64,
    pub ends_at: i64,
    pub status: GlobalLimitPreferenceStatus,
}

impl GlobalLimitPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalLimitPreferenceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalLimitPreferenceStatus) -> bool {
        match self.status {
            GlobalLimitPreferenceStatus::Created => {
                return next == GlobalLimitPreferenceStatus::Retired;
            }
            GlobalLimitPreferenceStatus::Validated => {
                match next {
                    GlobalLimitPreferenceStatus::Published | GlobalLimitPreferenceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalLimitPreferenceStatus::Published => {
                match next {
                    GlobalLimitPreferenceStatus::Retired => {
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

    pub fn move_to(&mut self, next: GlobalLimitPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global limit preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.discount + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalLimitPreference {
        GlobalLimitPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            archived: self.archived,
            rotation_days: self.rotation_days,
            duration_minutes: self.duration_minutes,
            window_days: self.window_days,
            discount: self.discount,
            capacity: self.capacity,
            rate: self.rate,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a global limit preference in status created.
pub fn new_global_limit_preference(code: &str, title: &str) -> GlobalLimitPreference {
    GlobalLimitPreference {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        archived: false,
        rotation_days: 0,
        duration_minutes: 0,
        window_days: 0,
        discount: 0.0,
        capacity: 0,
        rate: 0.0,
        ends_at: 0,
        status: GlobalLimitPreferenceStatus::Created,
    }
}
