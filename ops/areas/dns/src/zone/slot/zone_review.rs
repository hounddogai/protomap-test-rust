use super::*;

use acme_kit::text;

/// Where a zone review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneReviewStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ZoneReviewStatus {
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

impl std::fmt::Display for ZoneReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a zone review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneReview {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub margin: f64,
    pub summary: String,
    pub namespace: String,
    pub score: f64,
    pub status: ZoneReviewStatus,
}

impl ZoneReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneReviewStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneReviewStatus) -> bool {
        match self.status {
            ZoneReviewStatus::Planned => {
                return next == ZoneReviewStatus::Scheduled;
            }
            ZoneReviewStatus::Scheduled => {
                return next == ZoneReviewStatus::InProgress || next == ZoneReviewStatus::Cancelled;
            }
            ZoneReviewStatus::InProgress => {
                match next {
                    ZoneReviewStatus::Completed | ZoneReviewStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneReviewStatus::Completed => {
                match next {
                    ZoneReviewStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ZoneReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.cost - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneReview {
        ZoneReview {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            margin: self.margin,
            summary: self.summary.clone(),
            namespace: self.namespace.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a zone review in status planned.
pub fn new_zone_review(code: &str, name: &str) -> ZoneReview {
    ZoneReview {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        margin: 0.0,
        summary: "".to_string(),
        namespace: "".to_string(),
        score: 0.0,
        status: ZoneReviewStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_zone_review("code-632", "name-483");
        assert_eq!(item.status, ZoneReviewStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_zone_review("code-632", "name-483");
        if item.move_to(ZoneReviewStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneReviewStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_zone_review("code-632", "name-483");
        item.status = ZoneReviewStatus::Cancelled;
        assert!(
            item.move_to(ZoneReviewStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
