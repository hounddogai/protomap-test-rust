use super::*;

use acme_kit::text;

/// Where a region pin checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinCheckpointStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionPinCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionPinCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One region pin checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinCheckpoint {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub reviewer: String,
    pub ceiling: f64,
    pub budget: f64,
    pub archived: bool,
    pub discount: f64,
    pub category: String,
    pub burst: i64,
    pub approved: bool,
    pub status: RegionPinCheckpointStatus,
}

impl RegionPinCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinCheckpointStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinCheckpointStatus) -> bool {
        match self.status {
            RegionPinCheckpointStatus::Proposed => {
                match next {
                    RegionPinCheckpointStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinCheckpointStatus::Approved => {
                return next == RegionPinCheckpointStatus::Rejected || next == RegionPinCheckpointStatus::Withdrawn;
            }
            RegionPinCheckpointStatus::Rejected => {
                match next {
                    RegionPinCheckpointStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RegionPinCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.limit as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinCheckpoint {
        RegionPinCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            reviewer: self.reviewer.clone(),
            ceiling: self.ceiling,
            budget: self.budget,
            archived: self.archived,
            discount: self.discount,
            category: self.category.clone(),
            burst: self.burst,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a region pin checkpoint in status proposed.
pub fn new_region_pin_checkpoint(code: &str, title: &str) -> RegionPinCheckpoint {
    RegionPinCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        reviewer: "".to_string(),
        ceiling: 0.0,
        budget: 0.0,
        archived: false,
        discount: 0.0,
        category: "".to_string(),
        burst: 0,
        approved: false,
        status: RegionPinCheckpointStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_region_pin_checkpoint("code-323", "title-550");
        assert_eq!(item.status, RegionPinCheckpointStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_region_pin_checkpoint("code-323", "title-550");
        if item.move_to(RegionPinCheckpointStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinCheckpointStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_region_pin_checkpoint("code-323", "title-550");
        item.status = RegionPinCheckpointStatus::Withdrawn;
        assert!(
            item.move_to(RegionPinCheckpointStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
