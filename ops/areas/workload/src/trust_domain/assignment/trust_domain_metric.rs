use super::*;

use acme_kit::text;

/// Where a trust domain metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainMetricStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TrustDomainMetricStatus {
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

impl std::fmt::Display for TrustDomainMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a trust domain metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainMetric {
    pub code: String,
    pub label: String,
    pub ratio: f64,
    pub summary: String,
    pub weight: f64,
    pub status: TrustDomainMetricStatus,
}

impl TrustDomainMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainMetricStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainMetricStatus) -> bool {
        match self.status {
            TrustDomainMetricStatus::Planned => {
                match next {
                    TrustDomainMetricStatus::InProgress | TrustDomainMetricStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainMetricStatus::Scheduled => {
                match next {
                    TrustDomainMetricStatus::InProgress | TrustDomainMetricStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainMetricStatus::InProgress => {
                return next == TrustDomainMetricStatus::Completed;
            }
            TrustDomainMetricStatus::Completed => {
                match next {
                    TrustDomainMetricStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TrustDomainMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.weight - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainMetric {
        TrustDomainMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            ratio: self.ratio,
            summary: self.summary.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a trust domain metric in status planned.
pub fn new_trust_domain_metric(code: &str, label: &str) -> TrustDomainMetric {
    TrustDomainMetric {
        code: code.to_string(),
        label: label.to_string(),
        ratio: 0.0,
        summary: "".to_string(),
        weight: 0.0,
        status: TrustDomainMetricStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_trust_domain_metric("code-357", "label-572");
        assert_eq!(item.status, TrustDomainMetricStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_trust_domain_metric("code-357", "label-572");
        if item.move_to(TrustDomainMetricStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainMetricStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_trust_domain_metric("code-357", "label-572");
        item.status = TrustDomainMetricStatus::Cancelled;
        assert!(
            item.move_to(TrustDomainMetricStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
