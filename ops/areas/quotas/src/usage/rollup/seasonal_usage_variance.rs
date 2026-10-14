use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a seasonal usage variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalUsageVarianceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalUsageVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalUsageVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal usage variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalUsageVariance {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub channel: String,
    pub audience: String,
    pub created_at: i64,
    pub attributes: BTreeMap<String, String>,
    pub status: SeasonalUsageVarianceStatus,
}

impl SeasonalUsageVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalUsageVarianceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalUsageVarianceStatus) -> bool {
        match self.status {
            SeasonalUsageVarianceStatus::Proposed => {
                match next {
                    SeasonalUsageVarianceStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalUsageVarianceStatus::Approved => {
                match next {
                    SeasonalUsageVarianceStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalUsageVarianceStatus::Rejected => {
                return next == SeasonalUsageVarianceStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalUsageVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal usage variance {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
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

    pub fn adjusted_load(&self) -> f64 {
        ((self.created_at as f64 - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalUsageVariance {
        SeasonalUsageVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            channel: self.channel.clone(),
            audience: self.audience.clone(),
            created_at: self.created_at,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a seasonal usage variance in status proposed.
pub fn new_seasonal_usage_variance(code: &str, name: &str) -> SeasonalUsageVariance {
    SeasonalUsageVariance {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        channel: "".to_string(),
        audience: "".to_string(),
        created_at: 0,
        attributes: BTreeMap::new(),
        status: SeasonalUsageVarianceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_seasonal_usage_variance("code-705", "name-141");
        assert_eq!(item.status, SeasonalUsageVarianceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_seasonal_usage_variance("code-705", "name-141");
        if item.move_to(SeasonalUsageVarianceStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalUsageVarianceStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_seasonal_usage_variance("code-705", "name-141");
        item.status = SeasonalUsageVarianceStatus::Withdrawn;
        assert!(
            item.move_to(SeasonalUsageVarianceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
