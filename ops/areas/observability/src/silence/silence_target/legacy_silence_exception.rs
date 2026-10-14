use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a legacy silence exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacySilenceExceptionStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LegacySilenceExceptionStatus {
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

impl std::fmt::Display for LegacySilenceExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy silence exception as the silence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceException {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub priority: i64,
    pub margin: f64,
    pub attributes: BTreeMap<String, String>,
    pub checksum: String,
    pub status: LegacySilenceExceptionStatus,
}

impl LegacySilenceException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacySilenceExceptionStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacySilenceExceptionStatus) -> bool {
        match self.status {
            LegacySilenceExceptionStatus::Planned => {
                match next {
                    LegacySilenceExceptionStatus::Scheduled | LegacySilenceExceptionStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacySilenceExceptionStatus::Scheduled => {
                match next {
                    LegacySilenceExceptionStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacySilenceExceptionStatus::InProgress => {
                match next {
                    LegacySilenceExceptionStatus::Completed | LegacySilenceExceptionStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacySilenceExceptionStatus::Completed => {
                return next == LegacySilenceExceptionStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacySilenceExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy silence exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
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

    pub fn projected_volume(&self) -> f64 {
        ((self.priority as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacySilenceException {
        LegacySilenceException {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            priority: self.priority,
            margin: self.margin,
            attributes: self.attributes.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy silence exception in status planned.
pub fn new_legacy_silence_exception(code: &str, name: &str) -> LegacySilenceException {
    LegacySilenceException {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        priority: 0,
        margin: 0.0,
        attributes: BTreeMap::new(),
        checksum: "".to_string(),
        status: LegacySilenceExceptionStatus::Planned,
    }
}
