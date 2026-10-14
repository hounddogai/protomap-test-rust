use super::*;

use acme_kit::text;

/// Where a tiered release slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredReleaseSlotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TieredReleaseSlotStatus {
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

impl std::fmt::Display for TieredReleaseSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered release slot as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseSlot {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub unit: String,
    pub category: String,
    pub pinned: bool,
    pub score: f64,
    pub status: TieredReleaseSlotStatus,
}

impl TieredReleaseSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredReleaseSlotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredReleaseSlotStatus) -> bool {
        match self.status {
            TieredReleaseSlotStatus::Open => {
                match next {
                    TieredReleaseSlotStatus::InReview | TieredReleaseSlotStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredReleaseSlotStatus::InReview => {
                return next == TieredReleaseSlotStatus::Reopened;
            }
            TieredReleaseSlotStatus::Resolved => {
                return next == TieredReleaseSlotStatus::Archived;
            }
            TieredReleaseSlotStatus::Reopened => {
                match next {
                    TieredReleaseSlotStatus::Archived => {
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

    pub fn move_to(&mut self, next: TieredReleaseSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered release slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered release slot", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> TieredReleaseSlot {
        TieredReleaseSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            unit: self.unit.clone(),
            category: self.category.clone(),
            pinned: self.pinned,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a tiered release slot in status open.
pub fn new_tiered_release_slot(code: &str, name: &str) -> TieredReleaseSlot {
    TieredReleaseSlot {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        unit: "".to_string(),
        category: "".to_string(),
        pinned: false,
        score: 0.0,
        status: TieredReleaseSlotStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tiered_release_slot("code-829", "name-530");
        assert_eq!(item.status, TieredReleaseSlotStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_tiered_release_slot("code-829", "name-530");
        if item.move_to(TieredReleaseSlotStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredReleaseSlotStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tiered_release_slot("code-829", "name-530");
        item.status = TieredReleaseSlotStatus::Archived;
        assert!(
            item.move_to(TieredReleaseSlotStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
