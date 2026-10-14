use super::*;

use acme_kit::text;

/// Where a pending redaction adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRedactionAdjustmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PendingRedactionAdjustmentStatus {
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

impl std::fmt::Display for PendingRedactionAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending redaction adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionAdjustment {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub source: String,
    pub archived: bool,
    pub version: i64,
    pub quantity: i64,
    pub category: String,
    pub cluster: String,
    pub score: f64,
    pub owner_id: String,
    pub status: PendingRedactionAdjustmentStatus,
}

impl PendingRedactionAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRedactionAdjustmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRedactionAdjustmentStatus) -> bool {
        match self.status {
            PendingRedactionAdjustmentStatus::Open => {
                match next {
                    PendingRedactionAdjustmentStatus::Reopened | PendingRedactionAdjustmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRedactionAdjustmentStatus::InReview => {
                return next == PendingRedactionAdjustmentStatus::Reopened;
            }
            PendingRedactionAdjustmentStatus::Resolved => {
                match next {
                    PendingRedactionAdjustmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRedactionAdjustmentStatus::Reopened => {
                match next {
                    PendingRedactionAdjustmentStatus::Archived => {
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

    pub fn move_to(&mut self, next: PendingRedactionAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending redaction adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.score - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRedactionAdjustment {
        PendingRedactionAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            source: self.source.clone(),
            archived: self.archived,
            version: self.version,
            quantity: self.quantity,
            category: self.category.clone(),
            cluster: self.cluster.clone(),
            score: self.score,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a pending redaction adjustment in status open.
pub fn new_pending_redaction_adjustment(code: &str, title: &str) -> PendingRedactionAdjustment {
    PendingRedactionAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        source: "".to_string(),
        archived: false,
        version: 0,
        quantity: 0,
        category: "".to_string(),
        cluster: "".to_string(),
        score: 0.0,
        owner_id: "".to_string(),
        status: PendingRedactionAdjustmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_pending_redaction_adjustment("code-834", "title-176");
        assert_eq!(item.status, PendingRedactionAdjustmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_pending_redaction_adjustment("code-834", "title-176");
        if item.move_to(PendingRedactionAdjustmentStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingRedactionAdjustmentStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_pending_redaction_adjustment("code-834", "title-176");
        item.status = PendingRedactionAdjustmentStatus::Archived;
        assert!(
            item.move_to(PendingRedactionAdjustmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
