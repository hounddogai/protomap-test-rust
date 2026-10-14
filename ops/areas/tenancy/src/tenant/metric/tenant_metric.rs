use super::*;

use acme_kit::text;

/// Where a tenant metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantMetricStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TenantMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TenantMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tenant metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantMetric {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub created_at: i64,
    pub budget: f64,
    pub category: String,
    pub status: TenantMetricStatus,
}

impl TenantMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantMetricStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantMetricStatus) -> bool {
        match self.status {
            TenantMetricStatus::Requested => {
                match next {
                    TenantMetricStatus::Granted | TenantMetricStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantMetricStatus::Granted => {
                return next == TenantMetricStatus::Expired;
            }
            TenantMetricStatus::Revoked => {
                return next == TenantMetricStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TenantMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.budget + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantMetric {
        TenantMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            created_at: self.created_at,
            budget: self.budget,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a tenant metric in status requested.
pub fn new_tenant_metric(code: &str, label: &str) -> TenantMetric {
    TenantMetric {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        created_at: 0,
        budget: 0.0,
        category: "".to_string(),
        status: TenantMetricStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_tenant_metric("code-343", "label-838");
        assert_eq!(item.status, TenantMetricStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_tenant_metric("code-343", "label-838");
        if item.move_to(TenantMetricStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantMetricStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_tenant_metric("code-343", "label-838");
        item.status = TenantMetricStatus::Expired;
        assert!(
            item.move_to(TenantMetricStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
