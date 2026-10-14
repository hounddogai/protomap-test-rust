use super::*;

use acme_kit::text;

/// Where a plan review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanReviewStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PlanReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for PlanReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a plan review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanReview {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub audience: String,
    pub currency: String,
    pub source: String,
    pub cluster: String,
    pub priority: i64,
    pub country: String,
    pub status: PlanReviewStatus,
}

impl PlanReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanReviewStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanReviewStatus) -> bool {
        match self.status {
            PlanReviewStatus::Planned => {
                match next {
                    PlanReviewStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanReviewStatus::Scheduled => {
                match next {
                    PlanReviewStatus::Completed | PlanReviewStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanReviewStatus::InProgress => {
                match next {
                    PlanReviewStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanReviewStatus::Completed => {
                match next {
                    PlanReviewStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PlanReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.priority as f64) * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanReview {
        PlanReview {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            audience: self.audience.clone(),
            currency: self.currency.clone(),
            source: self.source.clone(),
            cluster: self.cluster.clone(),
            priority: self.priority,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a plan review in status planned.
pub fn new_plan_review(code: &str, title: &str) -> PlanReview {
    PlanReview {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        audience: "".to_string(),
        currency: "".to_string(),
        source: "".to_string(),
        cluster: "".to_string(),
        priority: 0,
        country: "".to_string(),
        status: PlanReviewStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_plan_review("code-611", "title-745");
        assert_eq!(item.status, PlanReviewStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_plan_review("code-611", "title-745");
        if item.move_to(PlanReviewStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanReviewStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_plan_review("code-611", "title-745");
        item.status = PlanReviewStatus::Cancelled;
        assert!(
            item.move_to(PlanReviewStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
