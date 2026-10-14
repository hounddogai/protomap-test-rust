use super::*;

use acme_kit::text;

/// Where a strict federation cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictFederationCycleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl StrictFederationCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for StrictFederationCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict federation cycle as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationCycle {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub created_at: i64,
    pub notes: String,
    pub checksum: String,
    pub summary: String,
    pub grace_days: i64,
    pub name: String,
    pub host: String,
    pub duration_minutes: i64,
    pub status: StrictFederationCycleStatus,
}

impl StrictFederationCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictFederationCycleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictFederationCycleStatus) -> bool {
        match self.status {
            StrictFederationCycleStatus::Proposed => {
                return next == StrictFederationCycleStatus::Rejected || next == StrictFederationCycleStatus::Withdrawn;
            }
            StrictFederationCycleStatus::Approved => {
                return next == StrictFederationCycleStatus::Rejected;
            }
            StrictFederationCycleStatus::Rejected => {
                return next == StrictFederationCycleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictFederationCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict federation cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.grace_days as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictFederationCycle {
        StrictFederationCycle {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            created_at: self.created_at,
            notes: self.notes.clone(),
            checksum: self.checksum.clone(),
            summary: self.summary.clone(),
            grace_days: self.grace_days,
            name: self.name.clone(),
            host: self.host.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a strict federation cycle in status proposed.
pub fn new_strict_federation_cycle(code: &str, title: &str) -> StrictFederationCycle {
    StrictFederationCycle {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        created_at: 0,
        notes: "".to_string(),
        checksum: "".to_string(),
        summary: "".to_string(),
        grace_days: 0,
        name: "".to_string(),
        host: "".to_string(),
        duration_minutes: 0,
        status: StrictFederationCycleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_strict_federation_cycle("code-332", "title-948");
        assert_eq!(item.status, StrictFederationCycleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_strict_federation_cycle("code-332", "title-948");
        if item.move_to(StrictFederationCycleStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictFederationCycleStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_strict_federation_cycle("code-332", "title-948");
        item.status = StrictFederationCycleStatus::Withdrawn;
        assert!(
            item.move_to(StrictFederationCycleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
