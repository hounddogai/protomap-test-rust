use super::*;

use acme_kit::text;

/// Where a evidence estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceEstimateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl EvidenceEstimateStatus {
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

impl std::fmt::Display for EvidenceEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A evidence estimate as the evidence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceEstimate {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub fingerprint: String,
    pub minimum: f64,
    pub currency: String,
    pub channel: String,
    pub sequence: i64,
    pub status: EvidenceEstimateStatus,
}

impl EvidenceEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceEstimateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceEstimateStatus) -> bool {
        match self.status {
            EvidenceEstimateStatus::Planned => {
                match next {
                    EvidenceEstimateStatus::Scheduled | EvidenceEstimateStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceEstimateStatus::Scheduled => {
                match next {
                    EvidenceEstimateStatus::InProgress | EvidenceEstimateStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceEstimateStatus::InProgress => {
                match next {
                    EvidenceEstimateStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceEstimateStatus::Completed => {
                match next {
                    EvidenceEstimateStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: EvidenceEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.minimum + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceEstimate {
        EvidenceEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            fingerprint: self.fingerprint.clone(),
            minimum: self.minimum,
            currency: self.currency.clone(),
            channel: self.channel.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a evidence estimate in status planned.
pub fn new_evidence_estimate(code: &str, title: &str) -> EvidenceEstimate {
    EvidenceEstimate {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        fingerprint: "".to_string(),
        minimum: 0.0,
        currency: "".to_string(),
        channel: "".to_string(),
        sequence: 0,
        status: EvidenceEstimateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_evidence_estimate("code-648", "title-795");
        assert_eq!(item.status, EvidenceEstimateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_evidence_estimate("code-648", "title-795");
        if item.move_to(EvidenceEstimateStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceEstimateStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_evidence_estimate("code-648", "title-795");
        item.status = EvidenceEstimateStatus::Cancelled;
        assert!(
            item.move_to(EvidenceEstimateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
