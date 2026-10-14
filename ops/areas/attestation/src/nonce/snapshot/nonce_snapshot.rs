use super::*;

use acme_kit::text;

/// Where a nonce snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceSnapshotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl NonceSnapshotStatus {
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

impl std::fmt::Display for NonceSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceSnapshot {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub unit: String,
    pub owner_id: String,
    pub ends_at: i64,
    pub rate: f64,
    pub timezone: String,
    pub source: String,
    pub capacity: i64,
    pub status: NonceSnapshotStatus,
}

impl NonceSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceSnapshotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceSnapshotStatus) -> bool {
        match self.status {
            NonceSnapshotStatus::Planned => {
                match next {
                    NonceSnapshotStatus::Scheduled | NonceSnapshotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceSnapshotStatus::Scheduled => {
                match next {
                    NonceSnapshotStatus::InProgress | NonceSnapshotStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceSnapshotStatus::InProgress => {
                return next == NonceSnapshotStatus::Completed || next == NonceSnapshotStatus::Cancelled;
            }
            NonceSnapshotStatus::Completed => {
                return next == NonceSnapshotStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: NonceSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "nonce snapshot", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.capacity as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceSnapshot {
        NonceSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            unit: self.unit.clone(),
            owner_id: self.owner_id.clone(),
            ends_at: self.ends_at,
            rate: self.rate,
            timezone: self.timezone.clone(),
            source: self.source.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a nonce snapshot in status planned.
pub fn new_nonce_snapshot(code: &str, name: &str) -> NonceSnapshot {
    NonceSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        unit: "".to_string(),
        owner_id: "".to_string(),
        ends_at: 0,
        rate: 0.0,
        timezone: "".to_string(),
        source: "".to_string(),
        capacity: 0,
        status: NonceSnapshotStatus::Planned,
    }
}
