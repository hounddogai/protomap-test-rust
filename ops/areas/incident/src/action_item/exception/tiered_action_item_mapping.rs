use super::*;

use acme_kit::text;

/// Where a tiered action item mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredActionItemMappingStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredActionItemMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredActionItemMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered action item mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredActionItemMapping {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub pinned: bool,
    pub window_days: i64,
    pub ends_at: i64,
    pub revision: i64,
    pub budget: f64,
    pub status: TieredActionItemMappingStatus,
}

impl TieredActionItemMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredActionItemMappingStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredActionItemMappingStatus) -> bool {
        match self.status {
            TieredActionItemMappingStatus::Proposed => {
                return next == TieredActionItemMappingStatus::Approved
                    || next == TieredActionItemMappingStatus::Rejected;
            }
            TieredActionItemMappingStatus::Approved => {
                match next {
                    TieredActionItemMappingStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredActionItemMappingStatus::Rejected => {
                return next == TieredActionItemMappingStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredActionItemMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered action item mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.revision as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredActionItemMapping {
        TieredActionItemMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            pinned: self.pinned,
            window_days: self.window_days,
            ends_at: self.ends_at,
            revision: self.revision,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a tiered action item mapping in status proposed.
pub fn new_tiered_action_item_mapping(code: &str, title: &str) -> TieredActionItemMapping {
    TieredActionItemMapping {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        pinned: false,
        window_days: 0,
        ends_at: 0,
        revision: 0,
        budget: 0.0,
        status: TieredActionItemMappingStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_tiered_action_item_mapping("code-103", "title-832");
        assert_eq!(item.status, TieredActionItemMappingStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_tiered_action_item_mapping("code-103", "title-832");
        if item.move_to(TieredActionItemMappingStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredActionItemMappingStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_tiered_action_item_mapping("code-103", "title-832");
        item.status = TieredActionItemMappingStatus::Withdrawn;
        assert!(
            item.move_to(TieredActionItemMappingStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
