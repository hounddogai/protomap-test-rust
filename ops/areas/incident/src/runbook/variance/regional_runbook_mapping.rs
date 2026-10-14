use super::*;

use acme_kit::text;

/// Where a regional runbook mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalRunbookMappingStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalRunbookMappingStatus {
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

impl std::fmt::Display for RegionalRunbookMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional runbook mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookMapping {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub key_id: String,
    pub ends_at: i64,
    pub status: RegionalRunbookMappingStatus,
}

impl RegionalRunbookMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalRunbookMappingStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalRunbookMappingStatus) -> bool {
        match self.status {
            RegionalRunbookMappingStatus::Idle => {
                return next == RegionalRunbookMappingStatus::Draining;
            }
            RegionalRunbookMappingStatus::Warming => {
                match next {
                    RegionalRunbookMappingStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRunbookMappingStatus::Ready => {
                return next == RegionalRunbookMappingStatus::Draining;
            }
            RegionalRunbookMappingStatus::Draining => {
                return next == RegionalRunbookMappingStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalRunbookMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional runbook mapping {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.ends_at as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalRunbookMapping {
        RegionalRunbookMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            key_id: self.key_id.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional runbook mapping in status idle.
pub fn new_regional_runbook_mapping(code: &str, name: &str) -> RegionalRunbookMapping {
    RegionalRunbookMapping {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        key_id: "".to_string(),
        ends_at: 0,
        status: RegionalRunbookMappingStatus::Idle,
    }
}
