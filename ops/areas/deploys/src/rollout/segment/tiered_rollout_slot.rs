use super::*;

use acme_kit::text;

/// Where a tiered rollout slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredRolloutSlotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredRolloutSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredRolloutSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered rollout slot as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutSlot {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub archived: bool,
    pub reviewer: String,
    pub ttl_seconds: i64,
    pub expires_at: i64,
    pub locale: String,
    pub name: String,
    pub floor_value: f64,
    pub starts_at: i64,
    pub status: TieredRolloutSlotStatus,
}

impl TieredRolloutSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredRolloutSlotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredRolloutSlotStatus) -> bool {
        match self.status {
            TieredRolloutSlotStatus::Proposed => {
                return next == TieredRolloutSlotStatus::Rejected || next == TieredRolloutSlotStatus::Withdrawn;
            }
            TieredRolloutSlotStatus::Approved => {
                return next == TieredRolloutSlotStatus::Rejected || next == TieredRolloutSlotStatus::Withdrawn;
            }
            TieredRolloutSlotStatus::Rejected => {
                match next {
                    TieredRolloutSlotStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TieredRolloutSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered rollout slot {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.expires_at as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredRolloutSlot {
        TieredRolloutSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            archived: self.archived,
            reviewer: self.reviewer.clone(),
            ttl_seconds: self.ttl_seconds,
            expires_at: self.expires_at,
            locale: self.locale.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a tiered rollout slot in status proposed.
pub fn new_tiered_rollout_slot(code: &str, title: &str) -> TieredRolloutSlot {
    TieredRolloutSlot {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        archived: false,
        reviewer: "".to_string(),
        ttl_seconds: 0,
        expires_at: 0,
        locale: "".to_string(),
        name: "".to_string(),
        floor_value: 0.0,
        starts_at: 0,
        status: TieredRolloutSlotStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_tiered_rollout_slot("code-62", "title-536");
        assert_eq!(item.status, TieredRolloutSlotStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_tiered_rollout_slot("code-62", "title-536");
        if item.move_to(TieredRolloutSlotStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredRolloutSlotStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_tiered_rollout_slot("code-62", "title-536");
        item.status = TieredRolloutSlotStatus::Withdrawn;
        assert!(
            item.move_to(TieredRolloutSlotStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
