use super::*;

use acme_kit::text;

/// Where a alert metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertMetricStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AlertMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AlertMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A alert metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertMetric {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub country: String,
    pub scope: String,
    pub floor_value: f64,
    pub sequence: i64,
    pub retries: i64,
    pub updated_at: i64,
    pub namespace: String,
    pub status: AlertMetricStatus,
}

impl AlertMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertMetricStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertMetricStatus) -> bool {
        match self.status {
            AlertMetricStatus::Created => {
                match next {
                    AlertMetricStatus::Validated | AlertMetricStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertMetricStatus::Validated => {
                match next {
                    AlertMetricStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertMetricStatus::Published => {
                return next == AlertMetricStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AlertMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "alert metric", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.floor_value - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertMetric {
        AlertMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            country: self.country.clone(),
            scope: self.scope.clone(),
            floor_value: self.floor_value,
            sequence: self.sequence,
            retries: self.retries,
            updated_at: self.updated_at,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a alert metric in status created.
pub fn new_alert_metric(code: &str, label: &str) -> AlertMetric {
    AlertMetric {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        country: "".to_string(),
        scope: "".to_string(),
        floor_value: 0.0,
        sequence: 0,
        retries: 0,
        updated_at: 0,
        namespace: "".to_string(),
        status: AlertMetricStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_alert_metric("code-651", "label-600");
        assert_eq!(item.status, AlertMetricStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_alert_metric("code-651", "label-600");
        if item.move_to(AlertMetricStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertMetricStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_alert_metric("code-651", "label-600");
        item.status = AlertMetricStatus::Retired;
        assert!(
            item.move_to(AlertMetricStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
