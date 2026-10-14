use super::*;

use acme_kit::text;

/// Where a strict release slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictReleaseSlotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictReleaseSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for StrictReleaseSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict release slot as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleaseSlot {
    pub code: String,
    pub title: String,
    pub cost: f64,
    pub limit: i64,
    pub active: bool,
    pub attempts: i64,
    pub unit: String,
    pub key_id: String,
    pub phone: String,
    pub audience: String,
    pub quota_units: i64,
    pub status: StrictReleaseSlotStatus,
}

impl StrictReleaseSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictReleaseSlotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictReleaseSlotStatus) -> bool {
        match self.status {
            StrictReleaseSlotStatus::Draft => {
                match next {
                    StrictReleaseSlotStatus::Suspended | StrictReleaseSlotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictReleaseSlotStatus::Pending => {
                return next == StrictReleaseSlotStatus::Closed;
            }
            StrictReleaseSlotStatus::Active => {
                match next {
                    StrictReleaseSlotStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictReleaseSlotStatus::Suspended => {
                return next == StrictReleaseSlotStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictReleaseSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict release slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict release slot", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.quota_units as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictReleaseSlot {
        StrictReleaseSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            cost: self.cost,
            limit: self.limit,
            active: self.active,
            attempts: self.attempts,
            unit: self.unit.clone(),
            key_id: self.key_id.clone(),
            phone: self.phone.clone(),
            audience: self.audience.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a strict release slot in status draft.
pub fn new_strict_release_slot(code: &str, title: &str) -> StrictReleaseSlot {
    StrictReleaseSlot {
        code: code.to_string(),
        title: title.to_string(),
        cost: 0.0,
        limit: 0,
        active: false,
        attempts: 0,
        unit: "".to_string(),
        key_id: "".to_string(),
        phone: "".to_string(),
        audience: "".to_string(),
        quota_units: 0,
        status: StrictReleaseSlotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_strict_release_slot("code-458", "title-902");
        assert_eq!(item.status, StrictReleaseSlotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_strict_release_slot("code-458", "title-902");
        if item.move_to(StrictReleaseSlotStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictReleaseSlotStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_strict_release_slot("code-458", "title-902");
        item.status = StrictReleaseSlotStatus::Closed;
        assert!(
            item.move_to(StrictReleaseSlotStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
