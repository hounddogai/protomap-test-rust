use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a rolling policy rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingPolicyRollupStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollingPolicyRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollingPolicyRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling policy rollup as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPolicyRollup {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub category: String,
    pub updated_at: i64,
    pub title: String,
    pub limit: i64,
    pub budget: f64,
    pub ends_at: i64,
    pub cost: f64,
    pub retries: i64,
    pub status: RollingPolicyRollupStatus,
}

impl RollingPolicyRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingPolicyRollupStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingPolicyRollupStatus) -> bool {
        match self.status {
            RollingPolicyRollupStatus::Requested => {
                return next == RollingPolicyRollupStatus::Revoked;
            }
            RollingPolicyRollupStatus::Granted => {
                return next == RollingPolicyRollupStatus::Revoked || next == RollingPolicyRollupStatus::Expired;
            }
            RollingPolicyRollupStatus::Revoked => {
                return next == RollingPolicyRollupStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingPolicyRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling policy rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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

    pub fn net_volume(&self) -> f64 {
        ((self.budget - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingPolicyRollup {
        RollingPolicyRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            category: self.category.clone(),
            updated_at: self.updated_at,
            title: self.title.clone(),
            limit: self.limit,
            budget: self.budget,
            ends_at: self.ends_at,
            cost: self.cost,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a rolling policy rollup in status requested.
pub fn new_rolling_policy_rollup(code: &str, label: &str) -> RollingPolicyRollup {
    RollingPolicyRollup {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        category: "".to_string(),
        updated_at: 0,
        title: "".to_string(),
        limit: 0,
        budget: 0.0,
        ends_at: 0,
        cost: 0.0,
        retries: 0,
        status: RollingPolicyRollupStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rolling_policy_rollup("code-283", "label-167");
        assert_eq!(item.status, RollingPolicyRollupStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_rolling_policy_rollup("code-283", "label-167");
        if item.move_to(RollingPolicyRollupStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingPolicyRollupStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rolling_policy_rollup("code-283", "label-167");
        item.status = RollingPolicyRollupStatus::Expired;
        assert!(
            item.move_to(RollingPolicyRollupStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
