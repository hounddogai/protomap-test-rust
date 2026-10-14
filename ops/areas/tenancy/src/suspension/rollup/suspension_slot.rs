use super::*;

use acme_kit::text;

/// Where a suspension slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionSlotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SuspensionSlotStatus {
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

impl std::fmt::Display for SuspensionSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension slot as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSlot {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub category: String,
    pub rotation_days: i64,
    pub attempts: i64,
    pub capacity: i64,
    pub threshold: f64,
    pub tags: Vec<String>,
    pub status: SuspensionSlotStatus,
}

impl SuspensionSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionSlotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionSlotStatus) -> bool {
        match self.status {
            SuspensionSlotStatus::Open => {
                match next {
                    SuspensionSlotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionSlotStatus::InReview => {
                return next == SuspensionSlotStatus::Reopened;
            }
            SuspensionSlotStatus::Resolved => {
                match next {
                    SuspensionSlotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionSlotStatus::Reopened => {
                match next {
                    SuspensionSlotStatus::Archived => {
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

    pub fn move_to(&mut self, next: SuspensionSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
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

    pub fn adjusted_volume(&self) -> f64 {
        ((self.capacity as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionSlot {
        SuspensionSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            category: self.category.clone(),
            rotation_days: self.rotation_days,
            attempts: self.attempts,
            capacity: self.capacity,
            threshold: self.threshold,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a suspension slot in status open.
pub fn new_suspension_slot(code: &str, name: &str) -> SuspensionSlot {
    SuspensionSlot {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        category: "".to_string(),
        rotation_days: 0,
        attempts: 0,
        capacity: 0,
        threshold: 0.0,
        tags: Vec::new(),
        status: SuspensionSlotStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_suspension_slot("code-512", "name-154");
        assert_eq!(item.status, SuspensionSlotStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_suspension_slot("code-512", "name-154");
        if item.move_to(SuspensionSlotStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionSlotStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_suspension_slot("code-512", "name-154");
        item.status = SuspensionSlotStatus::Archived;
        assert!(
            item.move_to(SuspensionSlotStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
