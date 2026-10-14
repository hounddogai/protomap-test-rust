use super::*;

use acme_kit::text;

/// Where a fallback limit metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackLimitMetricStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackLimitMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackLimitMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback limit metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackLimitMetric {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub starts_at: i64,
    pub scope: String,
    pub issuer: String,
    pub port: i64,
    pub quota_units: i64,
    pub summary: String,
    pub timezone: String,
    pub status: FallbackLimitMetricStatus,
}

impl FallbackLimitMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackLimitMetricStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackLimitMetricStatus) -> bool {
        match self.status {
            FallbackLimitMetricStatus::Requested => {
                return next == FallbackLimitMetricStatus::Revoked;
            }
            FallbackLimitMetricStatus::Granted => {
                match next {
                    FallbackLimitMetricStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackLimitMetricStatus::Revoked => {
                return next == FallbackLimitMetricStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackLimitMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback limit metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback limit metric", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.port as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackLimitMetric {
        FallbackLimitMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            starts_at: self.starts_at,
            scope: self.scope.clone(),
            issuer: self.issuer.clone(),
            port: self.port,
            quota_units: self.quota_units,
            summary: self.summary.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a fallback limit metric in status requested.
pub fn new_fallback_limit_metric(code: &str, name: &str) -> FallbackLimitMetric {
    FallbackLimitMetric {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        starts_at: 0,
        scope: "".to_string(),
        issuer: "".to_string(),
        port: 0,
        quota_units: 0,
        summary: "".to_string(),
        timezone: "".to_string(),
        status: FallbackLimitMetricStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_fallback_limit_metric("code-512", "name-110");
        assert_eq!(item.status, FallbackLimitMetricStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_fallback_limit_metric("code-512", "name-110");
        if item.move_to(FallbackLimitMetricStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackLimitMetricStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_fallback_limit_metric("code-512", "name-110");
        item.status = FallbackLimitMetricStatus::Expired;
        assert!(
            item.move_to(FallbackLimitMetricStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
