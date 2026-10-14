use super::*;

use acme_kit::text;

/// Where a overage slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageSlotStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl OverageSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for OverageSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A overage slot as the overage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageSlot {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub budget: f64,
    pub reviewer: String,
    pub revision: i64,
    pub key_id: String,
    pub country: String,
    pub status: OverageSlotStatus,
}

impl OverageSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageSlotStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageSlotStatus) -> bool {
        match self.status {
            OverageSlotStatus::Created => {
                return next == OverageSlotStatus::Validated;
            }
            OverageSlotStatus::Validated => {
                match next {
                    OverageSlotStatus::Published | OverageSlotStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageSlotStatus::Published => {
                match next {
                    OverageSlotStatus::Retired => {
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

    pub fn move_to(&mut self, next: OverageSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.budget - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageSlot {
        OverageSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            budget: self.budget,
            reviewer: self.reviewer.clone(),
            revision: self.revision,
            key_id: self.key_id.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a overage slot in status created.
pub fn new_overage_slot(code: &str, title: &str) -> OverageSlot {
    OverageSlot {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        budget: 0.0,
        reviewer: "".to_string(),
        revision: 0,
        key_id: "".to_string(),
        country: "".to_string(),
        status: OverageSlotStatus::Created,
    }
}
