use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a limit schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitScheduleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LimitScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for LimitScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One limit schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitSchedule {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub ceiling: f64,
    pub issuer: String,
    pub host: String,
    pub attributes: BTreeMap<String, String>,
    pub cost: f64,
    pub status: LimitScheduleStatus,
}

impl LimitSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitScheduleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitScheduleStatus) -> bool {
        match self.status {
            LimitScheduleStatus::Idle => {
                return next == LimitScheduleStatus::Warming || next == LimitScheduleStatus::Draining;
            }
            LimitScheduleStatus::Warming => {
                return next == LimitScheduleStatus::Ready || next == LimitScheduleStatus::Stopped;
            }
            LimitScheduleStatus::Ready => {
                return next == LimitScheduleStatus::Stopped;
            }
            LimitScheduleStatus::Draining => {
                return next == LimitScheduleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
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

    pub fn effective_load(&self) -> f64 {
        ((self.ceiling + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitSchedule {
        LimitSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            ceiling: self.ceiling,
            issuer: self.issuer.clone(),
            host: self.host.clone(),
            attributes: self.attributes.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a limit schedule in status idle.
pub fn new_limit_schedule(code: &str, label: &str) -> LimitSchedule {
    LimitSchedule {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        ceiling: 0.0,
        issuer: "".to_string(),
        host: "".to_string(),
        attributes: BTreeMap::new(),
        cost: 0.0,
        status: LimitScheduleStatus::Idle,
    }
}
