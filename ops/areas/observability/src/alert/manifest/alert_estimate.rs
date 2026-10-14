use super::*;

use acme_kit::text;

/// Where a alert estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertEstimateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AlertEstimateStatus {
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

impl std::fmt::Display for AlertEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a alert estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertEstimate {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub archived: bool,
    pub namespace: String,
    pub weight: f64,
    pub summary: String,
    pub contact_email: String,
    pub status: AlertEstimateStatus,
}

impl AlertEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertEstimateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertEstimateStatus) -> bool {
        match self.status {
            AlertEstimateStatus::Planned => {
                match next {
                    AlertEstimateStatus::Completed | AlertEstimateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertEstimateStatus::Scheduled => {
                return next == AlertEstimateStatus::InProgress;
            }
            AlertEstimateStatus::InProgress => {
                match next {
                    AlertEstimateStatus::Completed | AlertEstimateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertEstimateStatus::Completed => {
                match next {
                    AlertEstimateStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: AlertEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.weight
    }

    pub fn copy(&self) -> AlertEstimate {
        AlertEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            archived: self.archived,
            namespace: self.namespace.clone(),
            weight: self.weight,
            summary: self.summary.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a alert estimate in status planned.
pub fn new_alert_estimate(code: &str, title: &str) -> AlertEstimate {
    AlertEstimate {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        archived: false,
        namespace: "".to_string(),
        weight: 0.0,
        summary: "".to_string(),
        contact_email: "".to_string(),
        status: AlertEstimateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_alert_estimate("code-996", "title-402");
        assert_eq!(item.status, AlertEstimateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_alert_estimate("code-996", "title-402");
        if item.move_to(AlertEstimateStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertEstimateStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_alert_estimate("code-996", "title-402");
        item.status = AlertEstimateStatus::Cancelled;
        assert!(
            item.move_to(AlertEstimateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
