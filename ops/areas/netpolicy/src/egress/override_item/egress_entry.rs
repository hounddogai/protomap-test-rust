use super::*;

use acme_kit::text;

/// Where a egress entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressEntryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl EgressEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for EgressEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a egress entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressEntry {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub tags: Vec<String>,
    pub minimum: f64,
    pub ends_at: i64,
    pub status: EgressEntryStatus,
}

impl EgressEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressEntryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressEntryStatus) -> bool {
        match self.status {
            EgressEntryStatus::Created => {
                match next {
                    EgressEntryStatus::Validated | EgressEntryStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressEntryStatus::Validated => {
                match next {
                    EgressEntryStatus::Published | EgressEntryStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressEntryStatus::Published => {
                return next == EgressEntryStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress entry {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.ends_at as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressEntry {
        EgressEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            tags: self.tags.clone(),
            minimum: self.minimum,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a egress entry in status created.
pub fn new_egress_entry(code: &str, name: &str) -> EgressEntry {
    EgressEntry {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        tags: Vec::new(),
        minimum: 0.0,
        ends_at: 0,
        status: EgressEntryStatus::Created,
    }
}
