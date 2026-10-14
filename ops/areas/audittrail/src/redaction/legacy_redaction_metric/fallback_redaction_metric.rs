use super::*;

use acme_kit::text;

/// Where a fallback redaction metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRedactionMetricStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FallbackRedactionMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for FallbackRedactionMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback redaction metric as the redaction team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionMetric {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub score: f64,
    pub key_id: String,
    pub unit: String,
    pub host: String,
    pub attempts: i64,
    pub issuer: String,
    pub grace_days: i64,
    pub status: FallbackRedactionMetricStatus,
}

impl FallbackRedactionMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRedactionMetricStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRedactionMetricStatus) -> bool {
        match self.status {
            FallbackRedactionMetricStatus::Draft => {
                return next == FallbackRedactionMetricStatus::Pending;
            }
            FallbackRedactionMetricStatus::Pending => {
                match next {
                    FallbackRedactionMetricStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRedactionMetricStatus::Active => {
                return next == FallbackRedactionMetricStatus::Suspended
                    || next == FallbackRedactionMetricStatus::Closed;
            }
            FallbackRedactionMetricStatus::Suspended => {
                match next {
                    FallbackRedactionMetricStatus::Closed => {
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

    pub fn move_to(&mut self, next: FallbackRedactionMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback redaction metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback redaction metric", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.attempts as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackRedactionMetric {
        FallbackRedactionMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            score: self.score,
            key_id: self.key_id.clone(),
            unit: self.unit.clone(),
            host: self.host.clone(),
            attempts: self.attempts,
            issuer: self.issuer.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a fallback redaction metric in status draft.
pub fn new_fallback_redaction_metric(code: &str, name: &str) -> FallbackRedactionMetric {
    FallbackRedactionMetric {
        code: code.to_string(),
        name: name.to_string(),
        locale: "".to_string(),
        score: 0.0,
        key_id: "".to_string(),
        unit: "".to_string(),
        host: "".to_string(),
        attempts: 0,
        issuer: "".to_string(),
        grace_days: 0,
        status: FallbackRedactionMetricStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_fallback_redaction_metric("code-504", "name-466");
        assert_eq!(item.status, FallbackRedactionMetricStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_fallback_redaction_metric("code-504", "name-466");
        if item.move_to(FallbackRedactionMetricStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackRedactionMetricStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_fallback_redaction_metric("code-504", "name-466");
        item.status = FallbackRedactionMetricStatus::Closed;
        assert!(
            item.move_to(FallbackRedactionMetricStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
