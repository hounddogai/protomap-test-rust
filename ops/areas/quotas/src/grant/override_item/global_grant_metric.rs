use super::*;

use acme_kit::text;

/// Where a global grant metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalGrantMetricStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl GlobalGrantMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for GlobalGrantMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global grant metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalGrantMetric {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub segment: String,
    pub ttl_seconds: i64,
    pub summary: String,
    pub unit: String,
    pub weight: f64,
    pub timezone: String,
    pub namespace: String,
    pub status: GlobalGrantMetricStatus,
}

impl GlobalGrantMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalGrantMetricStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalGrantMetricStatus) -> bool {
        match self.status {
            GlobalGrantMetricStatus::Created => {
                return next == GlobalGrantMetricStatus::Validated;
            }
            GlobalGrantMetricStatus::Validated => {
                return next == GlobalGrantMetricStatus::Published;
            }
            GlobalGrantMetricStatus::Published => {
                return next == GlobalGrantMetricStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalGrantMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global grant metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.weight - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalGrantMetric {
        GlobalGrantMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            segment: self.segment.clone(),
            ttl_seconds: self.ttl_seconds,
            summary: self.summary.clone(),
            unit: self.unit.clone(),
            weight: self.weight,
            timezone: self.timezone.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a global grant metric in status created.
pub fn new_global_grant_metric(code: &str, name: &str) -> GlobalGrantMetric {
    GlobalGrantMetric {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        segment: "".to_string(),
        ttl_seconds: 0,
        summary: "".to_string(),
        unit: "".to_string(),
        weight: 0.0,
        timezone: "".to_string(),
        namespace: "".to_string(),
        status: GlobalGrantMetricStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_global_grant_metric("code-397", "name-491");
        assert_eq!(item.status, GlobalGrantMetricStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_global_grant_metric("code-397", "name-491");
        if item.move_to(GlobalGrantMetricStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalGrantMetricStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_global_grant_metric("code-397", "name-491");
        item.status = GlobalGrantMetricStatus::Retired;
        assert!(
            item.move_to(GlobalGrantMetricStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
