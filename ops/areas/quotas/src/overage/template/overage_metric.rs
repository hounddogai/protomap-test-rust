use super::*;

use acme_kit::text;

/// Where a overage metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageMetricStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl OverageMetricStatus {
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

impl std::fmt::Display for OverageMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A overage metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageMetric {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub description: String,
    pub cluster: String,
    pub status: OverageMetricStatus,
}

impl OverageMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageMetricStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageMetricStatus) -> bool {
        match self.status {
            OverageMetricStatus::Idle => {
                match next {
                    OverageMetricStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageMetricStatus::Warming => {
                return next == OverageMetricStatus::Draining || next == OverageMetricStatus::Stopped;
            }
            OverageMetricStatus::Ready => {
                return next == OverageMetricStatus::Draining || next == OverageMetricStatus::Stopped;
            }
            OverageMetricStatus::Draining => {
                match next {
                    OverageMetricStatus::Stopped => {
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

    pub fn move_to(&mut self, next: OverageMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "overage metric", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.minimum
    }

    pub fn copy(&self) -> OverageMetric {
        OverageMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            description: self.description.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a overage metric in status idle.
pub fn new_overage_metric(code: &str, label: &str) -> OverageMetric {
    OverageMetric {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        description: "".to_string(),
        cluster: "".to_string(),
        status: OverageMetricStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_overage_metric("code-650", "label-556");
        assert_eq!(item.status, OverageMetricStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_overage_metric("code-650", "label-556");
        if item.move_to(OverageMetricStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageMetricStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_overage_metric("code-650", "label-556");
        item.status = OverageMetricStatus::Stopped;
        assert!(
            item.move_to(OverageMetricStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
