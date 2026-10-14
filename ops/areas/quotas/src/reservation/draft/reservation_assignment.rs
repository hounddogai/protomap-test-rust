use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a reservation assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationAssignmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ReservationAssignmentStatus {
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

impl std::fmt::Display for ReservationAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation assignment as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAssignment {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub updated_at: i64,
    pub revision: i64,
    pub attributes: BTreeMap<String, String>,
    pub label: String,
    pub region: String,
    pub burst: i64,
    pub locale: String,
    pub priority: i64,
    pub status: ReservationAssignmentStatus,
}

impl ReservationAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationAssignmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationAssignmentStatus) -> bool {
        match self.status {
            ReservationAssignmentStatus::Planned => {
                return next == ReservationAssignmentStatus::InProgress;
            }
            ReservationAssignmentStatus::Scheduled => {
                match next {
                    ReservationAssignmentStatus::InProgress | ReservationAssignmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationAssignmentStatus::InProgress => {
                match next {
                    ReservationAssignmentStatus::Completed | ReservationAssignmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationAssignmentStatus::Completed => {
                match next {
                    ReservationAssignmentStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ReservationAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "reservation assignment", self.code)
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

    pub fn effective_exposure(&self) -> f64 {
        ((self.updated_at as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationAssignment {
        ReservationAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            updated_at: self.updated_at,
            revision: self.revision,
            attributes: self.attributes.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            burst: self.burst,
            locale: self.locale.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a reservation assignment in status planned.
pub fn new_reservation_assignment(code: &str, name: &str) -> ReservationAssignment {
    ReservationAssignment {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        updated_at: 0,
        revision: 0,
        attributes: BTreeMap::new(),
        label: "".to_string(),
        region: "".to_string(),
        burst: 0,
        locale: "".to_string(),
        priority: 0,
        status: ReservationAssignmentStatus::Planned,
    }
}
