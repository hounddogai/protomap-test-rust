use super::*;

use acme_kit::text;

/// Where a overage review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageReviewStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl OverageReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for OverageReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A overage review as the overage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageReview {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub ends_at: i64,
    pub amount: f64,
    pub external_ref: String,
    pub visible: bool,
    pub created_at: i64,
    pub cluster: String,
    pub label: String,
    pub status: OverageReviewStatus,
}

impl OverageReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageReviewStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageReviewStatus) -> bool {
        match self.status {
            OverageReviewStatus::New => {
                match next {
                    OverageReviewStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageReviewStatus::Queued => {
                return next == OverageReviewStatus::Done || next == OverageReviewStatus::Failed;
            }
            OverageReviewStatus::Running => {
                return next == OverageReviewStatus::Done || next == OverageReviewStatus::Failed;
            }
            OverageReviewStatus::Done => {
                match next {
                    OverageReviewStatus::Failed => {
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

    pub fn move_to(&mut self, next: OverageReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "overage review", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.ends_at as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageReview {
        OverageReview {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            ends_at: self.ends_at,
            amount: self.amount,
            external_ref: self.external_ref.clone(),
            visible: self.visible,
            created_at: self.created_at,
            cluster: self.cluster.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a overage review in status new.
pub fn new_overage_review(code: &str, name: &str) -> OverageReview {
    OverageReview {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        ends_at: 0,
        amount: 0.0,
        external_ref: "".to_string(),
        visible: false,
        created_at: 0,
        cluster: "".to_string(),
        label: "".to_string(),
        status: OverageReviewStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_overage_review("code-481", "name-507");
        assert_eq!(item.status, OverageReviewStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_overage_review("code-481", "name-507");
        if item.move_to(OverageReviewStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageReviewStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_overage_review("code-481", "name-507");
        item.status = OverageReviewStatus::Failed;
        assert!(item.move_to(OverageReviewStatus::New).is_err(), "expected move to fail");
    }
}
