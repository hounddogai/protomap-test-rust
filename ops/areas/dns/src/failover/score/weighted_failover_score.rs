use super::*;

use acme_kit::text;

/// Where a weighted failover score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedFailoverScoreStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedFailoverScoreStatus {
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

impl std::fmt::Display for WeightedFailoverScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted failover score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFailoverScore {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub quantity: i64,
    pub country: String,
    pub reviewer: String,
    pub maximum: f64,
    pub priority: i64,
    pub unit: String,
    pub status: WeightedFailoverScoreStatus,
}

impl WeightedFailoverScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedFailoverScoreStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedFailoverScoreStatus) -> bool {
        match self.status {
            WeightedFailoverScoreStatus::Planned => {
                match next {
                    WeightedFailoverScoreStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedFailoverScoreStatus::Scheduled => {
                return next == WeightedFailoverScoreStatus::Completed
                    || next == WeightedFailoverScoreStatus::Cancelled;
            }
            WeightedFailoverScoreStatus::InProgress => {
                match next {
                    WeightedFailoverScoreStatus::Completed | WeightedFailoverScoreStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedFailoverScoreStatus::Completed => {
                return next == WeightedFailoverScoreStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedFailoverScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted failover score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted failover score", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.maximum - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedFailoverScore {
        WeightedFailoverScore {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            quantity: self.quantity,
            country: self.country.clone(),
            reviewer: self.reviewer.clone(),
            maximum: self.maximum,
            priority: self.priority,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a weighted failover score in status planned.
pub fn new_weighted_failover_score(code: &str, title: &str) -> WeightedFailoverScore {
    WeightedFailoverScore {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        quantity: 0,
        country: "".to_string(),
        reviewer: "".to_string(),
        maximum: 0.0,
        priority: 0,
        unit: "".to_string(),
        status: WeightedFailoverScoreStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_weighted_failover_score("code-670", "title-386");
        assert_eq!(item.status, WeightedFailoverScoreStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_weighted_failover_score("code-670", "title-386");
        if item.move_to(WeightedFailoverScoreStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedFailoverScoreStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_weighted_failover_score("code-670", "title-386");
        item.status = WeightedFailoverScoreStatus::Cancelled;
        assert!(
            item.move_to(WeightedFailoverScoreStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
