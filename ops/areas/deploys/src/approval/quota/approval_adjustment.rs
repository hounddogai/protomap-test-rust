use super::*;

use acme_kit::text;

/// Where a approval adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalAdjustmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ApprovalAdjustmentStatus {
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

impl std::fmt::Display for ApprovalAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval adjustment as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAdjustment {
    pub code: String,
    pub title: String,
    pub country: String,
    pub visible: bool,
    pub unit: String,
    pub score: f64,
    pub weight: f64,
    pub host: String,
    pub amount: f64,
    pub cost: f64,
    pub discount: f64,
    pub status: ApprovalAdjustmentStatus,
}

impl ApprovalAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalAdjustmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalAdjustmentStatus) -> bool {
        match self.status {
            ApprovalAdjustmentStatus::New => {
                return next == ApprovalAdjustmentStatus::Queued || next == ApprovalAdjustmentStatus::Done;
            }
            ApprovalAdjustmentStatus::Queued => {
                match next {
                    ApprovalAdjustmentStatus::Running | ApprovalAdjustmentStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalAdjustmentStatus::Running => {
                return next == ApprovalAdjustmentStatus::Done || next == ApprovalAdjustmentStatus::Failed;
            }
            ApprovalAdjustmentStatus::Done => {
                match next {
                    ApprovalAdjustmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: ApprovalAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.score - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalAdjustment {
        ApprovalAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            visible: self.visible,
            unit: self.unit.clone(),
            score: self.score,
            weight: self.weight,
            host: self.host.clone(),
            amount: self.amount,
            cost: self.cost,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a approval adjustment in status new.
pub fn new_approval_adjustment(code: &str, title: &str) -> ApprovalAdjustment {
    ApprovalAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        visible: false,
        unit: "".to_string(),
        score: 0.0,
        weight: 0.0,
        host: "".to_string(),
        amount: 0.0,
        cost: 0.0,
        discount: 0.0,
        status: ApprovalAdjustmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_approval_adjustment("code-138", "title-27");
        assert_eq!(item.status, ApprovalAdjustmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_approval_adjustment("code-138", "title-27");
        if item.move_to(ApprovalAdjustmentStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalAdjustmentStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_approval_adjustment("code-138", "title-27");
        item.status = ApprovalAdjustmentStatus::Failed;
        assert!(
            item.move_to(ApprovalAdjustmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
