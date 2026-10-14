use super::*;

use acme_kit::text;

/// Where a suspension group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionGroupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SuspensionGroupStatus {
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

impl std::fmt::Display for SuspensionGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension group as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionGroup {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub quantity: i64,
    pub priority: i64,
    pub batch_size: i64,
    pub score: f64,
    pub status: SuspensionGroupStatus,
}

impl SuspensionGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionGroupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionGroupStatus) -> bool {
        match self.status {
            SuspensionGroupStatus::Open => {
                return next == SuspensionGroupStatus::InReview || next == SuspensionGroupStatus::Archived;
            }
            SuspensionGroupStatus::InReview => {
                match next {
                    SuspensionGroupStatus::Resolved | SuspensionGroupStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionGroupStatus::Resolved => {
                return next == SuspensionGroupStatus::Reopened || next == SuspensionGroupStatus::Archived;
            }
            SuspensionGroupStatus::Reopened => {
                return next == SuspensionGroupStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "suspension group", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.batch_size as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionGroup {
        SuspensionGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            quantity: self.quantity,
            priority: self.priority,
            batch_size: self.batch_size,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a suspension group in status open.
pub fn new_suspension_group(code: &str, title: &str) -> SuspensionGroup {
    SuspensionGroup {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        quantity: 0,
        priority: 0,
        batch_size: 0,
        score: 0.0,
        status: SuspensionGroupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_suspension_group("code-873", "title-955");
        assert_eq!(item.status, SuspensionGroupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_suspension_group("code-873", "title-955");
        if item.move_to(SuspensionGroupStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionGroupStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_suspension_group("code-873", "title-955");
        item.status = SuspensionGroupStatus::Archived;
        assert!(
            item.move_to(SuspensionGroupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
