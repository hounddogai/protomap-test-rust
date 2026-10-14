use super::*;

use acme_kit::text;

/// Where a rule metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleMetricStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RuleMetricStatus {
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

impl std::fmt::Display for RuleMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rule metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleMetric {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub cluster: String,
    pub quota_units: i64,
    pub cost: f64,
    pub limit: i64,
    pub grace_days: i64,
    pub namespace: String,
    pub status: RuleMetricStatus,
}

impl RuleMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleMetricStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleMetricStatus) -> bool {
        match self.status {
            RuleMetricStatus::Idle => {
                return next == RuleMetricStatus::Warming || next == RuleMetricStatus::Draining;
            }
            RuleMetricStatus::Warming => {
                match next {
                    RuleMetricStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleMetricStatus::Ready => {
                match next {
                    RuleMetricStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleMetricStatus::Draining => {
                match next {
                    RuleMetricStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RuleMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rule metric", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_value(&self) -> f64 {
        ((self.grace_days as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleMetric {
        RuleMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            cluster: self.cluster.clone(),
            quota_units: self.quota_units,
            cost: self.cost,
            limit: self.limit,
            grace_days: self.grace_days,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a rule metric in status idle.
pub fn new_rule_metric(code: &str, label: &str) -> RuleMetric {
    RuleMetric {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        cluster: "".to_string(),
        quota_units: 0,
        cost: 0.0,
        limit: 0,
        grace_days: 0,
        namespace: "".to_string(),
        status: RuleMetricStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_rule_metric("code-399", "label-484");
        assert_eq!(item.status, RuleMetricStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_rule_metric("code-399", "label-484");
        if item.move_to(RuleMetricStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleMetricStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_rule_metric("code-399", "label-484");
        item.status = RuleMetricStatus::Stopped;
        assert!(item.move_to(RuleMetricStatus::Idle).is_err(), "expected move to fail");
    }
}
