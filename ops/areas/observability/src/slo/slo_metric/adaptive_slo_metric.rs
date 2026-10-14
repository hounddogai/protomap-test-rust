use super::*;

use acme_kit::text;

/// Where a adaptive slo metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveSloMetricStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AdaptiveSloMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AdaptiveSloMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive slo metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloMetric {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub tenant_id: String,
    pub floor_value: f64,
    pub tags: Vec<String>,
    pub quota_units: i64,
    pub currency: String,
    pub owner_id: String,
    pub status: AdaptiveSloMetricStatus,
}

impl AdaptiveSloMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveSloMetricStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveSloMetricStatus) -> bool {
        match self.status {
            AdaptiveSloMetricStatus::Requested => {
                return next == AdaptiveSloMetricStatus::Revoked || next == AdaptiveSloMetricStatus::Expired;
            }
            AdaptiveSloMetricStatus::Granted => {
                match next {
                    AdaptiveSloMetricStatus::Revoked | AdaptiveSloMetricStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveSloMetricStatus::Revoked => {
                return next == AdaptiveSloMetricStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveSloMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive slo metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_total(&self) -> f64 {
        ((self.quota_units as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveSloMetric {
        AdaptiveSloMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            tenant_id: self.tenant_id.clone(),
            floor_value: self.floor_value,
            tags: self.tags.clone(),
            quota_units: self.quota_units,
            currency: self.currency.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a adaptive slo metric in status requested.
pub fn new_adaptive_slo_metric(code: &str, title: &str) -> AdaptiveSloMetric {
    AdaptiveSloMetric {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        tenant_id: "".to_string(),
        floor_value: 0.0,
        tags: Vec::new(),
        quota_units: 0,
        currency: "".to_string(),
        owner_id: "".to_string(),
        status: AdaptiveSloMetricStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_adaptive_slo_metric("code-630", "title-248");
        assert_eq!(item.status, AdaptiveSloMetricStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_adaptive_slo_metric("code-630", "title-248");
        if item.move_to(AdaptiveSloMetricStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveSloMetricStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_adaptive_slo_metric("code-630", "title-248");
        item.status = AdaptiveSloMetricStatus::Expired;
        assert!(
            item.move_to(AdaptiveSloMetricStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
