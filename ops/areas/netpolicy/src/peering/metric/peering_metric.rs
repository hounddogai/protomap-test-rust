use super::*;

use acme_kit::text;

/// Where a peering metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringMetricStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PeeringMetricStatus {
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

impl std::fmt::Display for PeeringMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A peering metric as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringMetric {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub category: String,
    pub source: String,
    pub discount: f64,
    pub budget: f64,
    pub status: PeeringMetricStatus,
}

impl PeeringMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringMetricStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringMetricStatus) -> bool {
        match self.status {
            PeeringMetricStatus::Idle => {
                match next {
                    PeeringMetricStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringMetricStatus::Warming => {
                return next == PeeringMetricStatus::Draining || next == PeeringMetricStatus::Stopped;
            }
            PeeringMetricStatus::Ready => {
                match next {
                    PeeringMetricStatus::Draining | PeeringMetricStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringMetricStatus::Draining => {
                match next {
                    PeeringMetricStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PeeringMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.discount + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringMetric {
        PeeringMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            category: self.category.clone(),
            source: self.source.clone(),
            discount: self.discount,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a peering metric in status idle.
pub fn new_peering_metric(code: &str, name: &str) -> PeeringMetric {
    PeeringMetric {
        code: code.to_string(),
        name: name.to_string(),
        currency: "".to_string(),
        category: "".to_string(),
        source: "".to_string(),
        discount: 0.0,
        budget: 0.0,
        status: PeeringMetricStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_peering_metric("code-9", "name-578");
        assert_eq!(item.status, PeeringMetricStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_peering_metric("code-9", "name-578");
        if item.move_to(PeeringMetricStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringMetricStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_peering_metric("code-9", "name-578");
        item.status = PeeringMetricStatus::Stopped;
        assert!(
            item.move_to(PeeringMetricStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
