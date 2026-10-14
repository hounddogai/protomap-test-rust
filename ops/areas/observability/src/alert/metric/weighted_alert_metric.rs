use super::*;

use acme_kit::text;

/// Where a weighted alert metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedAlertMetricStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl WeightedAlertMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for WeightedAlertMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted alert metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertMetric {
    pub code: String,
    pub label: String,
    pub sequence: i64,
    pub revision: i64,
    pub burst: i64,
    pub status: WeightedAlertMetricStatus,
}

impl WeightedAlertMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedAlertMetricStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedAlertMetricStatus) -> bool {
        match self.status {
            WeightedAlertMetricStatus::Created => {
                match next {
                    WeightedAlertMetricStatus::Published | WeightedAlertMetricStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedAlertMetricStatus::Validated => {
                return next == WeightedAlertMetricStatus::Retired;
            }
            WeightedAlertMetricStatus::Published => {
                return next == WeightedAlertMetricStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedAlertMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted alert metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.sequence as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedAlertMetric {
        WeightedAlertMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            sequence: self.sequence,
            revision: self.revision,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted alert metric in status created.
pub fn new_weighted_alert_metric(code: &str, label: &str) -> WeightedAlertMetric {
    WeightedAlertMetric {
        code: code.to_string(),
        label: label.to_string(),
        sequence: 0,
        revision: 0,
        burst: 0,
        status: WeightedAlertMetricStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_weighted_alert_metric("code-97", "label-350");
        assert_eq!(item.status, WeightedAlertMetricStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_weighted_alert_metric("code-97", "label-350");
        if item.move_to(WeightedAlertMetricStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedAlertMetricStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_weighted_alert_metric("code-97", "label-350");
        item.status = WeightedAlertMetricStatus::Retired;
        assert!(
            item.move_to(WeightedAlertMetricStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
