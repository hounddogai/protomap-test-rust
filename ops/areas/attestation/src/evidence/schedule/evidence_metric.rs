use super::*;

use acme_kit::text;

/// Where a evidence metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceMetricStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl EvidenceMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for EvidenceMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One evidence metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceMetric {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub ratio: f64,
    pub amount: f64,
    pub port: i64,
    pub maximum: f64,
    pub capacity: i64,
    pub currency: String,
    pub status: EvidenceMetricStatus,
}

impl EvidenceMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceMetricStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceMetricStatus) -> bool {
        match self.status {
            EvidenceMetricStatus::Idle => {
                match next {
                    EvidenceMetricStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceMetricStatus::Warming => {
                match next {
                    EvidenceMetricStatus::Ready | EvidenceMetricStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceMetricStatus::Ready => {
                match next {
                    EvidenceMetricStatus::Draining | EvidenceMetricStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceMetricStatus::Draining => {
                match next {
                    EvidenceMetricStatus::Stopped => {
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

    pub fn move_to(&mut self, next: EvidenceMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "evidence metric", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceMetric {
        EvidenceMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            ratio: self.ratio,
            amount: self.amount,
            port: self.port,
            maximum: self.maximum,
            capacity: self.capacity,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a evidence metric in status idle.
pub fn new_evidence_metric(code: &str, title: &str) -> EvidenceMetric {
    EvidenceMetric {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        ratio: 0.0,
        amount: 0.0,
        port: 0,
        maximum: 0.0,
        capacity: 0,
        currency: "".to_string(),
        status: EvidenceMetricStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_evidence_metric("code-610", "title-38");
        assert_eq!(item.status, EvidenceMetricStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_evidence_metric("code-610", "title-38");
        if item.move_to(EvidenceMetricStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceMetricStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_evidence_metric("code-610", "title-38");
        item.status = EvidenceMetricStatus::Stopped;
        assert!(
            item.move_to(EvidenceMetricStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
