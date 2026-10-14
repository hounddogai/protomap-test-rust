use super::*;

use acme_kit::text;

/// Where a escalation metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationMetricStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl EscalationMetricStatus {
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

impl std::fmt::Display for EscalationMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a escalation metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMetric {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub scope: String,
    pub priority: i64,
    pub quota_units: i64,
    pub budget: f64,
    pub pinned: bool,
    pub external_ref: String,
    pub source: String,
    pub discount: f64,
    pub status: EscalationMetricStatus,
}

impl EscalationMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationMetricStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationMetricStatus) -> bool {
        match self.status {
            EscalationMetricStatus::New => {
                match next {
                    EscalationMetricStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationMetricStatus::Queued => {
                return next == EscalationMetricStatus::Running || next == EscalationMetricStatus::Failed;
            }
            EscalationMetricStatus::Running => {
                match next {
                    EscalationMetricStatus::Done | EscalationMetricStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationMetricStatus::Done => {
                return next == EscalationMetricStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EscalationMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "escalation metric", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.priority as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationMetric {
        EscalationMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            scope: self.scope.clone(),
            priority: self.priority,
            quota_units: self.quota_units,
            budget: self.budget,
            pinned: self.pinned,
            external_ref: self.external_ref.clone(),
            source: self.source.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a escalation metric in status new.
pub fn new_escalation_metric(code: &str, name: &str) -> EscalationMetric {
    EscalationMetric {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        scope: "".to_string(),
        priority: 0,
        quota_units: 0,
        budget: 0.0,
        pinned: false,
        external_ref: "".to_string(),
        source: "".to_string(),
        discount: 0.0,
        status: EscalationMetricStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_escalation_metric("code-724", "name-409");
        assert_eq!(item.status, EscalationMetricStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_escalation_metric("code-724", "name-409");
        if item.move_to(EscalationMetricStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationMetricStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_escalation_metric("code-724", "name-409");
        item.status = EscalationMetricStatus::Failed;
        assert!(
            item.move_to(EscalationMetricStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
