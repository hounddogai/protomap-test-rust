use super::*;

use acme_kit::text;

/// Where a weighted identity record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedIdentityRecordStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedIdentityRecordStatus {
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

impl std::fmt::Display for WeightedIdentityRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted identity record and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityRecord {
    pub code: String,
    pub label: String,
    pub category: String,
    pub visible: bool,
    pub ends_at: i64,
    pub status: WeightedIdentityRecordStatus,
}

impl WeightedIdentityRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedIdentityRecordStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedIdentityRecordStatus) -> bool {
        match self.status {
            WeightedIdentityRecordStatus::Planned => {
                match next {
                    WeightedIdentityRecordStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedIdentityRecordStatus::Scheduled => {
                match next {
                    WeightedIdentityRecordStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedIdentityRecordStatus::InProgress => {
                match next {
                    WeightedIdentityRecordStatus::Completed | WeightedIdentityRecordStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedIdentityRecordStatus::Completed => {
                match next {
                    WeightedIdentityRecordStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: WeightedIdentityRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted identity record {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> WeightedIdentityRecord {
        WeightedIdentityRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            visible: self.visible,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a weighted identity record in status planned.
pub fn new_weighted_identity_record(code: &str, label: &str) -> WeightedIdentityRecord {
    WeightedIdentityRecord {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        visible: false,
        ends_at: 0,
        status: WeightedIdentityRecordStatus::Planned,
    }
}
