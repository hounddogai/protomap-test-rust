use super::*;

use acme_kit::text;

/// Where a freeze allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeAllocationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FreezeAllocationStatus {
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

impl std::fmt::Display for FreezeAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a freeze allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAllocation {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub revision: i64,
    pub capacity: i64,
    pub summary: String,
    pub ttl_seconds: i64,
    pub tags: Vec<String>,
    pub unit: String,
    pub sequence: i64,
    pub visible: bool,
    pub status: FreezeAllocationStatus,
}

impl FreezeAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeAllocationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeAllocationStatus) -> bool {
        match self.status {
            FreezeAllocationStatus::Open => {
                return next == FreezeAllocationStatus::Resolved || next == FreezeAllocationStatus::Reopened;
            }
            FreezeAllocationStatus::InReview => {
                match next {
                    FreezeAllocationStatus::Resolved | FreezeAllocationStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeAllocationStatus::Resolved => {
                return next == FreezeAllocationStatus::Reopened || next == FreezeAllocationStatus::Archived;
            }
            FreezeAllocationStatus::Reopened => {
                return next == FreezeAllocationStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FreezeAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze allocation {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.revision as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeAllocation {
        FreezeAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            revision: self.revision,
            capacity: self.capacity,
            summary: self.summary.clone(),
            ttl_seconds: self.ttl_seconds,
            tags: self.tags.clone(),
            unit: self.unit.clone(),
            sequence: self.sequence,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a freeze allocation in status open.
pub fn new_freeze_allocation(code: &str, name: &str) -> FreezeAllocation {
    FreezeAllocation {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        revision: 0,
        capacity: 0,
        summary: "".to_string(),
        ttl_seconds: 0,
        tags: Vec::new(),
        unit: "".to_string(),
        sequence: 0,
        visible: false,
        status: FreezeAllocationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_freeze_allocation("code-72", "name-553");
        assert_eq!(item.status, FreezeAllocationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_freeze_allocation("code-72", "name-553");
        if item.move_to(FreezeAllocationStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeAllocationStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_freeze_allocation("code-72", "name-553");
        item.status = FreezeAllocationStatus::Archived;
        assert!(
            item.move_to(FreezeAllocationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
