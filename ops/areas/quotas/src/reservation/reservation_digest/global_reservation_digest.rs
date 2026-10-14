use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a global reservation digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalReservationDigestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalReservationDigestStatus {
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

impl std::fmt::Display for GlobalReservationDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global reservation digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReservationDigest {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub active: bool,
    pub scope: String,
    pub notes: String,
    pub attributes: BTreeMap<String, String>,
    pub locale: String,
    pub audience: String,
    pub port: i64,
    pub priority: i64,
    pub status: GlobalReservationDigestStatus,
}

impl GlobalReservationDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalReservationDigestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalReservationDigestStatus) -> bool {
        match self.status {
            GlobalReservationDigestStatus::Planned => {
                return next == GlobalReservationDigestStatus::InProgress;
            }
            GlobalReservationDigestStatus::Scheduled => {
                match next {
                    GlobalReservationDigestStatus::Completed | GlobalReservationDigestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalReservationDigestStatus::InProgress => {
                return next == GlobalReservationDigestStatus::Completed
                    || next == GlobalReservationDigestStatus::Cancelled;
            }
            GlobalReservationDigestStatus::Completed => {
                return next == GlobalReservationDigestStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalReservationDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global reservation digest {} cannot move from {} to {}",
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

    pub fn adjusted_volume(&self) -> f64 {
        ((self.threshold + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalReservationDigest {
        GlobalReservationDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            active: self.active,
            scope: self.scope.clone(),
            notes: self.notes.clone(),
            attributes: self.attributes.clone(),
            locale: self.locale.clone(),
            audience: self.audience.clone(),
            port: self.port,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a global reservation digest in status planned.
pub fn new_global_reservation_digest(code: &str, name: &str) -> GlobalReservationDigest {
    GlobalReservationDigest {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        active: false,
        scope: "".to_string(),
        notes: "".to_string(),
        attributes: BTreeMap::new(),
        locale: "".to_string(),
        audience: "".to_string(),
        port: 0,
        priority: 0,
        status: GlobalReservationDigestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_global_reservation_digest("code-709", "name-467");
        assert_eq!(item.status, GlobalReservationDigestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_global_reservation_digest("code-709", "name-467");
        if item.move_to(GlobalReservationDigestStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalReservationDigestStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_global_reservation_digest("code-709", "name-467");
        item.status = GlobalReservationDigestStatus::Cancelled;
        assert!(
            item.move_to(GlobalReservationDigestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
