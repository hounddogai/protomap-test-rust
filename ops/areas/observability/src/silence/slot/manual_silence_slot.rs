use super::*;

use acme_kit::text;

/// Where a manual silence slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualSilenceSlotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ManualSilenceSlotStatus {
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

impl std::fmt::Display for ManualSilenceSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual silence slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceSlot {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub phone: String,
    pub limit: i64,
    pub rotation_days: i64,
    pub capacity: i64,
    pub approved: bool,
    pub cluster: String,
    pub minimum: f64,
    pub status: ManualSilenceSlotStatus,
}

impl ManualSilenceSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualSilenceSlotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualSilenceSlotStatus) -> bool {
        match self.status {
            ManualSilenceSlotStatus::Draft => {
                match next {
                    ManualSilenceSlotStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualSilenceSlotStatus::Pending => {
                match next {
                    ManualSilenceSlotStatus::Active | ManualSilenceSlotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualSilenceSlotStatus::Active => {
                return next == ManualSilenceSlotStatus::Suspended || next == ManualSilenceSlotStatus::Closed;
            }
            ManualSilenceSlotStatus::Suspended => {
                match next {
                    ManualSilenceSlotStatus::Closed => {
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

    pub fn move_to(&mut self, next: ManualSilenceSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual silence slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.rotation_days as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualSilenceSlot {
        ManualSilenceSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            phone: self.phone.clone(),
            limit: self.limit,
            rotation_days: self.rotation_days,
            capacity: self.capacity,
            approved: self.approved,
            cluster: self.cluster.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a manual silence slot in status draft.
pub fn new_manual_silence_slot(code: &str, name: &str) -> ManualSilenceSlot {
    ManualSilenceSlot {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        phone: "".to_string(),
        limit: 0,
        rotation_days: 0,
        capacity: 0,
        approved: false,
        cluster: "".to_string(),
        minimum: 0.0,
        status: ManualSilenceSlotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_manual_silence_slot("code-167", "name-45");
        assert_eq!(item.status, ManualSilenceSlotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_manual_silence_slot("code-167", "name-45");
        if item.move_to(ManualSilenceSlotStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualSilenceSlotStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_manual_silence_slot("code-167", "name-45");
        item.status = ManualSilenceSlotStatus::Closed;
        assert!(
            item.move_to(ManualSilenceSlotStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
