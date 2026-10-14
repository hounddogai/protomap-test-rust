use super::*;

use acme_kit::text;

/// Where a scheduled measurement notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledMeasurementNoticeStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ScheduledMeasurementNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ScheduledMeasurementNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled measurement notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledMeasurementNotice {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub tags: Vec<String>,
    pub country: String,
    pub score: f64,
    pub status: ScheduledMeasurementNoticeStatus,
}

impl ScheduledMeasurementNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledMeasurementNoticeStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledMeasurementNoticeStatus) -> bool {
        match self.status {
            ScheduledMeasurementNoticeStatus::Requested => {
                match next {
                    ScheduledMeasurementNoticeStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledMeasurementNoticeStatus::Granted => {
                return next == ScheduledMeasurementNoticeStatus::Revoked
                    || next == ScheduledMeasurementNoticeStatus::Expired;
            }
            ScheduledMeasurementNoticeStatus::Revoked => {
                match next {
                    ScheduledMeasurementNoticeStatus::Expired => {
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

    pub fn move_to(&mut self, next: ScheduledMeasurementNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled measurement notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled measurement notice", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> ScheduledMeasurementNotice {
        ScheduledMeasurementNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            tags: self.tags.clone(),
            country: self.country.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a scheduled measurement notice in status requested.
pub fn new_scheduled_measurement_notice(code: &str, name: &str) -> ScheduledMeasurementNotice {
    ScheduledMeasurementNotice {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        tags: Vec::new(),
        country: "".to_string(),
        score: 0.0,
        status: ScheduledMeasurementNoticeStatus::Requested,
    }
}
