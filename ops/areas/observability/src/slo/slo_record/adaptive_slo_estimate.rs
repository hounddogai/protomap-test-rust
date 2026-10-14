use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a adaptive slo estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveSloEstimateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AdaptiveSloEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AdaptiveSloEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive slo estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloEstimate {
    pub code: String,
    pub label: String,
    pub source: String,
    pub attributes: BTreeMap<String, String>,
    pub locale: String,
    pub duration_minutes: i64,
    pub port: i64,
    pub margin: f64,
    pub starts_at: i64,
    pub weight: f64,
    pub score: f64,
    pub status: AdaptiveSloEstimateStatus,
}

impl AdaptiveSloEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveSloEstimateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveSloEstimateStatus) -> bool {
        match self.status {
            AdaptiveSloEstimateStatus::Requested => {
                match next {
                    AdaptiveSloEstimateStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveSloEstimateStatus::Granted => {
                return next == AdaptiveSloEstimateStatus::Revoked;
            }
            AdaptiveSloEstimateStatus::Revoked => {
                match next {
                    AdaptiveSloEstimateStatus::Expired => {
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

    pub fn move_to(&mut self, next: AdaptiveSloEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive slo estimate {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive slo estimate", self.code)
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

    pub fn net_value(&self) -> f64 {
        (self.weight * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveSloEstimate {
        AdaptiveSloEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            attributes: self.attributes.clone(),
            locale: self.locale.clone(),
            duration_minutes: self.duration_minutes,
            port: self.port,
            margin: self.margin,
            starts_at: self.starts_at,
            weight: self.weight,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a adaptive slo estimate in status requested.
pub fn new_adaptive_slo_estimate(code: &str, label: &str) -> AdaptiveSloEstimate {
    AdaptiveSloEstimate {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        attributes: BTreeMap::new(),
        locale: "".to_string(),
        duration_minutes: 0,
        port: 0,
        margin: 0.0,
        starts_at: 0,
        weight: 0.0,
        score: 0.0,
        status: AdaptiveSloEstimateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_adaptive_slo_estimate("code-327", "label-956");
        assert_eq!(item.status, AdaptiveSloEstimateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_adaptive_slo_estimate("code-327", "label-956");
        if item.move_to(AdaptiveSloEstimateStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveSloEstimateStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_adaptive_slo_estimate("code-327", "label-956");
        item.status = AdaptiveSloEstimateStatus::Expired;
        assert!(
            item.move_to(AdaptiveSloEstimateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
