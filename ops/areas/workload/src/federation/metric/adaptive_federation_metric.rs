use super::*;

use acme_kit::text;

/// Where a adaptive federation metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveFederationMetricStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AdaptiveFederationMetricStatus {
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

impl std::fmt::Display for AdaptiveFederationMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive federation metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFederationMetric {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub created_at: i64,
    pub retries: i64,
    pub unit: String,
    pub checksum: String,
    pub rate: f64,
    pub status: AdaptiveFederationMetricStatus,
}

impl AdaptiveFederationMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveFederationMetricStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveFederationMetricStatus) -> bool {
        match self.status {
            AdaptiveFederationMetricStatus::Draft => {
                match next {
                    AdaptiveFederationMetricStatus::Suspended | AdaptiveFederationMetricStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveFederationMetricStatus::Pending => {
                return next == AdaptiveFederationMetricStatus::Active
                    || next == AdaptiveFederationMetricStatus::Suspended;
            }
            AdaptiveFederationMetricStatus::Active => {
                match next {
                    AdaptiveFederationMetricStatus::Suspended | AdaptiveFederationMetricStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveFederationMetricStatus::Suspended => {
                return next == AdaptiveFederationMetricStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveFederationMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive federation metric {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.created_at as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveFederationMetric {
        AdaptiveFederationMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            created_at: self.created_at,
            retries: self.retries,
            unit: self.unit.clone(),
            checksum: self.checksum.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a adaptive federation metric in status draft.
pub fn new_adaptive_federation_metric(code: &str, title: &str) -> AdaptiveFederationMetric {
    AdaptiveFederationMetric {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        created_at: 0,
        retries: 0,
        unit: "".to_string(),
        checksum: "".to_string(),
        rate: 0.0,
        status: AdaptiveFederationMetricStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_adaptive_federation_metric("code-576", "title-800");
        assert_eq!(item.status, AdaptiveFederationMetricStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_adaptive_federation_metric("code-576", "title-800");
        if item.move_to(AdaptiveFederationMetricStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveFederationMetricStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_adaptive_federation_metric("code-576", "title-800");
        item.status = AdaptiveFederationMetricStatus::Closed;
        assert!(
            item.move_to(AdaptiveFederationMetricStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
