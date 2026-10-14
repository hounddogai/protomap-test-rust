use super::*;

use acme_kit::text;

/// Where a primary limit metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryLimitMetricStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PrimaryLimitMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PrimaryLimitMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary limit metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitMetric {
    pub code: String,
    pub label: String,
    pub budget: f64,
    pub pinned: bool,
    pub namespace: String,
    pub status: PrimaryLimitMetricStatus,
}

impl PrimaryLimitMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryLimitMetricStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryLimitMetricStatus) -> bool {
        match self.status {
            PrimaryLimitMetricStatus::Created => {
                return next == PrimaryLimitMetricStatus::Validated || next == PrimaryLimitMetricStatus::Published;
            }
            PrimaryLimitMetricStatus::Validated => {
                match next {
                    PrimaryLimitMetricStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryLimitMetricStatus::Published => {
                return next == PrimaryLimitMetricStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryLimitMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary limit metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.budget
    }

    pub fn copy(&self) -> PrimaryLimitMetric {
        PrimaryLimitMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            budget: self.budget,
            pinned: self.pinned,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a primary limit metric in status created.
pub fn new_primary_limit_metric(code: &str, label: &str) -> PrimaryLimitMetric {
    PrimaryLimitMetric {
        code: code.to_string(),
        label: label.to_string(),
        budget: 0.0,
        pinned: false,
        namespace: "".to_string(),
        status: PrimaryLimitMetricStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_primary_limit_metric("code-232", "label-75");
        assert_eq!(item.status, PrimaryLimitMetricStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_primary_limit_metric("code-232", "label-75");
        if item.move_to(PrimaryLimitMetricStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryLimitMetricStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_primary_limit_metric("code-232", "label-75");
        item.status = PrimaryLimitMetricStatus::Retired;
        assert!(
            item.move_to(PrimaryLimitMetricStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
