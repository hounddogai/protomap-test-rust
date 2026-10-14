use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a primary rollout estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRolloutEstimateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryRolloutEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryRolloutEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary rollout estimate as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutEstimate {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub attributes: BTreeMap<String, String>,
    pub approved: bool,
    pub source: String,
    pub burst: i64,
    pub ceiling: f64,
    pub status: PrimaryRolloutEstimateStatus,
}

impl PrimaryRolloutEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRolloutEstimateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRolloutEstimateStatus) -> bool {
        match self.status {
            PrimaryRolloutEstimateStatus::Requested => {
                match next {
                    PrimaryRolloutEstimateStatus::Revoked | PrimaryRolloutEstimateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryRolloutEstimateStatus::Granted => {
                return next == PrimaryRolloutEstimateStatus::Expired;
            }
            PrimaryRolloutEstimateStatus::Revoked => {
                match next {
                    PrimaryRolloutEstimateStatus::Expired => {
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

    pub fn move_to(&mut self, next: PrimaryRolloutEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary rollout estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
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

    pub fn adjusted_volume(&self) -> f64 {
        ((self.burst as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryRolloutEstimate {
        PrimaryRolloutEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            cipher: self.cipher.clone(),
            attributes: self.attributes.clone(),
            approved: self.approved,
            source: self.source.clone(),
            burst: self.burst,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a primary rollout estimate in status requested.
pub fn new_primary_rollout_estimate(code: &str, name: &str) -> PrimaryRolloutEstimate {
    PrimaryRolloutEstimate {
        code: code.to_string(),
        name: name.to_string(),
        cipher: "".to_string(),
        attributes: BTreeMap::new(),
        approved: false,
        source: "".to_string(),
        burst: 0,
        ceiling: 0.0,
        status: PrimaryRolloutEstimateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_primary_rollout_estimate("code-708", "name-775");
        assert_eq!(item.status, PrimaryRolloutEstimateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_primary_rollout_estimate("code-708", "name-775");
        if item.move_to(PrimaryRolloutEstimateStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryRolloutEstimateStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_primary_rollout_estimate("code-708", "name-775");
        item.status = PrimaryRolloutEstimateStatus::Expired;
        assert!(
            item.move_to(PrimaryRolloutEstimateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
