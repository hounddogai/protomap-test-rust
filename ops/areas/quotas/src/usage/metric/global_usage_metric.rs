use super::*;

use acme_kit::text;

/// Where a global usage metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalUsageMetricStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GlobalUsageMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GlobalUsageMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global usage metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUsageMetric {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub category: String,
    pub issuer: String,
    pub phone: String,
    pub status: GlobalUsageMetricStatus,
}

impl GlobalUsageMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalUsageMetricStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalUsageMetricStatus) -> bool {
        match self.status {
            GlobalUsageMetricStatus::Requested => {
                return next == GlobalUsageMetricStatus::Revoked;
            }
            GlobalUsageMetricStatus::Granted => {
                match next {
                    GlobalUsageMetricStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalUsageMetricStatus::Revoked => {
                return next == GlobalUsageMetricStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalUsageMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global usage metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global usage metric", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.ratio
    }

    pub fn copy(&self) -> GlobalUsageMetric {
        GlobalUsageMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            category: self.category.clone(),
            issuer: self.issuer.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a global usage metric in status requested.
pub fn new_global_usage_metric(code: &str, name: &str) -> GlobalUsageMetric {
    GlobalUsageMetric {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        category: "".to_string(),
        issuer: "".to_string(),
        phone: "".to_string(),
        status: GlobalUsageMetricStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_global_usage_metric("code-279", "name-545");
        assert_eq!(item.status, GlobalUsageMetricStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_global_usage_metric("code-279", "name-545");
        if item.move_to(GlobalUsageMetricStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalUsageMetricStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_global_usage_metric("code-279", "name-545");
        item.status = GlobalUsageMetricStatus::Expired;
        assert!(
            item.move_to(GlobalUsageMetricStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
