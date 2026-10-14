use super::*;

use acme_kit::text;

/// Where a federation slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationSlotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FederationSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for FederationSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a federation slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationSlot {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub attempts: i64,
    pub expires_at: i64,
    pub contact_email: String,
    pub notes: String,
    pub sequence: i64,
    pub source: String,
    pub audience: String,
    pub status: FederationSlotStatus,
}

impl FederationSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationSlotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationSlotStatus) -> bool {
        match self.status {
            FederationSlotStatus::New => {
                return next == FederationSlotStatus::Failed;
            }
            FederationSlotStatus::Queued => {
                return next == FederationSlotStatus::Done;
            }
            FederationSlotStatus::Running => {
                return next == FederationSlotStatus::Done || next == FederationSlotStatus::Failed;
            }
            FederationSlotStatus::Done => {
                return next == FederationSlotStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.expires_at as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationSlot {
        FederationSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            attempts: self.attempts,
            expires_at: self.expires_at,
            contact_email: self.contact_email.clone(),
            notes: self.notes.clone(),
            sequence: self.sequence,
            source: self.source.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a federation slot in status new.
pub fn new_federation_slot(code: &str, name: &str) -> FederationSlot {
    FederationSlot {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        attempts: 0,
        expires_at: 0,
        contact_email: "".to_string(),
        notes: "".to_string(),
        sequence: 0,
        source: "".to_string(),
        audience: "".to_string(),
        status: FederationSlotStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_federation_slot("code-398", "name-925");
        assert_eq!(item.status, FederationSlotStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_federation_slot("code-398", "name-925");
        if item.move_to(FederationSlotStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationSlotStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_federation_slot("code-398", "name-925");
        item.status = FederationSlotStatus::Failed;
        assert!(
            item.move_to(FederationSlotStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
