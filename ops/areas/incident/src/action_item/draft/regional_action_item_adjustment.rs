use super::*;

use acme_kit::text;

/// Where a regional action item adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalActionItemAdjustmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalActionItemAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for RegionalActionItemAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional action item adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemAdjustment {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub weight: f64,
    pub sequence: i64,
    pub cipher: String,
    pub active: bool,
    pub status: RegionalActionItemAdjustmentStatus,
}

impl RegionalActionItemAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalActionItemAdjustmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalActionItemAdjustmentStatus) -> bool {
        match self.status {
            RegionalActionItemAdjustmentStatus::Open => {
                match next {
                    RegionalActionItemAdjustmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalActionItemAdjustmentStatus::InReview => {
                match next {
                    RegionalActionItemAdjustmentStatus::Resolved | RegionalActionItemAdjustmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalActionItemAdjustmentStatus::Resolved => {
                match next {
                    RegionalActionItemAdjustmentStatus::Reopened | RegionalActionItemAdjustmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalActionItemAdjustmentStatus::Reopened => {
                match next {
                    RegionalActionItemAdjustmentStatus::Archived => {
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

    pub fn move_to(&mut self, next: RegionalActionItemAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional action item adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.sequence as f64) * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalActionItemAdjustment {
        RegionalActionItemAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            weight: self.weight,
            sequence: self.sequence,
            cipher: self.cipher.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional action item adjustment in status open.
pub fn new_regional_action_item_adjustment(code: &str, name: &str) -> RegionalActionItemAdjustment {
    RegionalActionItemAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        weight: 0.0,
        sequence: 0,
        cipher: "".to_string(),
        active: false,
        status: RegionalActionItemAdjustmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_regional_action_item_adjustment("code-823", "name-926");
        assert_eq!(item.status, RegionalActionItemAdjustmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_regional_action_item_adjustment("code-823", "name-926");
        if item.move_to(RegionalActionItemAdjustmentStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalActionItemAdjustmentStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_regional_action_item_adjustment("code-823", "name-926");
        item.status = RegionalActionItemAdjustmentStatus::Archived;
        assert!(
            item.move_to(RegionalActionItemAdjustmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
