use super::*;

use acme_kit::text;

/// Where a nonce slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceSlotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl NonceSlotStatus {
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

impl std::fmt::Display for NonceSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One nonce slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceSlot {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub reviewer: String,
    pub window_days: i64,
    pub retries: i64,
    pub contact_email: String,
    pub version: i64,
    pub revision: i64,
    pub grace_days: i64,
    pub starts_at: i64,
    pub status: NonceSlotStatus,
}

impl NonceSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceSlotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceSlotStatus) -> bool {
        match self.status {
            NonceSlotStatus::Planned => {
                return next == NonceSlotStatus::Scheduled || next == NonceSlotStatus::Cancelled;
            }
            NonceSlotStatus::Scheduled => {
                match next {
                    NonceSlotStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceSlotStatus::InProgress => {
                match next {
                    NonceSlotStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceSlotStatus::Completed => {
                match next {
                    NonceSlotStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: NonceSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.starts_at as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceSlot {
        NonceSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            reviewer: self.reviewer.clone(),
            window_days: self.window_days,
            retries: self.retries,
            contact_email: self.contact_email.clone(),
            version: self.version,
            revision: self.revision,
            grace_days: self.grace_days,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a nonce slot in status planned.
pub fn new_nonce_slot(code: &str, name: &str) -> NonceSlot {
    NonceSlot {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        reviewer: "".to_string(),
        window_days: 0,
        retries: 0,
        contact_email: "".to_string(),
        version: 0,
        revision: 0,
        grace_days: 0,
        starts_at: 0,
        status: NonceSlotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_nonce_slot("code-229", "name-425");
        assert_eq!(item.status, NonceSlotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_nonce_slot("code-229", "name-425");
        if item.move_to(NonceSlotStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceSlotStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_nonce_slot("code-229", "name-425");
        item.status = NonceSlotStatus::Cancelled;
        assert!(item.move_to(NonceSlotStatus::Planned).is_err(), "expected move to fail");
    }
}
