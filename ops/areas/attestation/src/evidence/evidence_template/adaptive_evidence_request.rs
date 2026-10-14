use super::*;

use acme_kit::text;

/// Where a adaptive evidence request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveEvidenceRequestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AdaptiveEvidenceRequestStatus {
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

impl std::fmt::Display for AdaptiveEvidenceRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive evidence request as the evidence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEvidenceRequest {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub title: String,
    pub visible: bool,
    pub minimum: f64,
    pub maximum: f64,
    pub window_days: i64,
    pub ratio: f64,
    pub summary: String,
    pub notes: String,
    pub status: AdaptiveEvidenceRequestStatus,
}

impl AdaptiveEvidenceRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveEvidenceRequestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveEvidenceRequestStatus) -> bool {
        match self.status {
            AdaptiveEvidenceRequestStatus::Planned => {
                match next {
                    AdaptiveEvidenceRequestStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveEvidenceRequestStatus::Scheduled => {
                match next {
                    AdaptiveEvidenceRequestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveEvidenceRequestStatus::InProgress => {
                return next == AdaptiveEvidenceRequestStatus::Cancelled;
            }
            AdaptiveEvidenceRequestStatus::Completed => {
                return next == AdaptiveEvidenceRequestStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveEvidenceRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive evidence request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive evidence request", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.ratio - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveEvidenceRequest {
        AdaptiveEvidenceRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            title: self.title.clone(),
            visible: self.visible,
            minimum: self.minimum,
            maximum: self.maximum,
            window_days: self.window_days,
            ratio: self.ratio,
            summary: self.summary.clone(),
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a adaptive evidence request in status planned.
pub fn new_adaptive_evidence_request(code: &str, label: &str) -> AdaptiveEvidenceRequest {
    AdaptiveEvidenceRequest {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        title: "".to_string(),
        visible: false,
        minimum: 0.0,
        maximum: 0.0,
        window_days: 0,
        ratio: 0.0,
        summary: "".to_string(),
        notes: "".to_string(),
        status: AdaptiveEvidenceRequestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_adaptive_evidence_request("code-471", "label-5");
        assert_eq!(item.status, AdaptiveEvidenceRequestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_adaptive_evidence_request("code-471", "label-5");
        if item.move_to(AdaptiveEvidenceRequestStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveEvidenceRequestStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_adaptive_evidence_request("code-471", "label-5");
        item.status = AdaptiveEvidenceRequestStatus::Cancelled;
        assert!(
            item.move_to(AdaptiveEvidenceRequestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
