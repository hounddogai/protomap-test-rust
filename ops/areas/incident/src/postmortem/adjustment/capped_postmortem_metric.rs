use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a capped postmortem metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedPostmortemMetricStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CappedPostmortemMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CappedPostmortemMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped postmortem metric as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPostmortemMetric {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub attributes: BTreeMap<String, String>,
    pub audience: String,
    pub category: String,
    pub region: String,
    pub revision: i64,
    pub window_days: i64,
    pub score: f64,
    pub status: CappedPostmortemMetricStatus,
}

impl CappedPostmortemMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedPostmortemMetricStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedPostmortemMetricStatus) -> bool {
        match self.status {
            CappedPostmortemMetricStatus::Requested => {
                return next == CappedPostmortemMetricStatus::Granted;
            }
            CappedPostmortemMetricStatus::Granted => {
                return next == CappedPostmortemMetricStatus::Revoked;
            }
            CappedPostmortemMetricStatus::Revoked => {
                match next {
                    CappedPostmortemMetricStatus::Expired => {
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

    pub fn move_to(&mut self, next: CappedPostmortemMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped postmortem metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_total(&self) -> f64 {
        (self.score * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedPostmortemMetric {
        CappedPostmortemMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            attributes: self.attributes.clone(),
            audience: self.audience.clone(),
            category: self.category.clone(),
            region: self.region.clone(),
            revision: self.revision,
            window_days: self.window_days,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a capped postmortem metric in status requested.
pub fn new_capped_postmortem_metric(code: &str, name: &str) -> CappedPostmortemMetric {
    CappedPostmortemMetric {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        attributes: BTreeMap::new(),
        audience: "".to_string(),
        category: "".to_string(),
        region: "".to_string(),
        revision: 0,
        window_days: 0,
        score: 0.0,
        status: CappedPostmortemMetricStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_capped_postmortem_metric("code-842", "name-55");
        assert_eq!(item.status, CappedPostmortemMetricStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_capped_postmortem_metric("code-842", "name-55");
        if item.move_to(CappedPostmortemMetricStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedPostmortemMetricStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_capped_postmortem_metric("code-842", "name-55");
        item.status = CappedPostmortemMetricStatus::Expired;
        assert!(
            item.move_to(CappedPostmortemMetricStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
