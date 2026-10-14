use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a redaction schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionScheduleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RedactionScheduleStatus {
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

impl std::fmt::Display for RedactionScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One redaction schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionSchedule {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub cipher: String,
    pub unit: String,
    pub port: i64,
    pub score: f64,
    pub budget: f64,
    pub region: String,
    pub status: RedactionScheduleStatus,
}

impl RedactionSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionScheduleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionScheduleStatus) -> bool {
        match self.status {
            RedactionScheduleStatus::New => {
                return next == RedactionScheduleStatus::Done || next == RedactionScheduleStatus::Failed;
            }
            RedactionScheduleStatus::Queued => {
                return next == RedactionScheduleStatus::Running || next == RedactionScheduleStatus::Done;
            }
            RedactionScheduleStatus::Running => {
                return next == RedactionScheduleStatus::Done;
            }
            RedactionScheduleStatus::Done => {
                return next == RedactionScheduleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RedactionScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_load(&self) -> f64 {
        (self.budget * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionSchedule {
        RedactionSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            cipher: self.cipher.clone(),
            unit: self.unit.clone(),
            port: self.port,
            score: self.score,
            budget: self.budget,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a redaction schedule in status new.
pub fn new_redaction_schedule(code: &str, label: &str) -> RedactionSchedule {
    RedactionSchedule {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        cipher: "".to_string(),
        unit: "".to_string(),
        port: 0,
        score: 0.0,
        budget: 0.0,
        region: "".to_string(),
        status: RedactionScheduleStatus::New,
    }
}
