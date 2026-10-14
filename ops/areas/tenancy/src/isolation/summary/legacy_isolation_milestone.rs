use super::*;

use acme_kit::text;

/// Where a legacy isolation milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyIsolationMilestoneStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LegacyIsolationMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LegacyIsolationMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy isolation milestone as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationMilestone {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub version: i64,
    pub ceiling: f64,
    pub window_days: i64,
    pub quota_units: i64,
    pub key_id: String,
    pub minimum: f64,
    pub status: LegacyIsolationMilestoneStatus,
}

impl LegacyIsolationMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyIsolationMilestoneStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyIsolationMilestoneStatus) -> bool {
        match self.status {
            LegacyIsolationMilestoneStatus::Requested => {
                match next {
                    LegacyIsolationMilestoneStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyIsolationMilestoneStatus::Granted => {
                match next {
                    LegacyIsolationMilestoneStatus::Revoked | LegacyIsolationMilestoneStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyIsolationMilestoneStatus::Revoked => {
                match next {
                    LegacyIsolationMilestoneStatus::Expired => {
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

    pub fn move_to(&mut self, next: LegacyIsolationMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy isolation milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.version as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyIsolationMilestone {
        LegacyIsolationMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            version: self.version,
            ceiling: self.ceiling,
            window_days: self.window_days,
            quota_units: self.quota_units,
            key_id: self.key_id.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy isolation milestone in status requested.
pub fn new_legacy_isolation_milestone(code: &str, name: &str) -> LegacyIsolationMilestone {
    LegacyIsolationMilestone {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        version: 0,
        ceiling: 0.0,
        window_days: 0,
        quota_units: 0,
        key_id: "".to_string(),
        minimum: 0.0,
        status: LegacyIsolationMilestoneStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_legacy_isolation_milestone("code-369", "name-983");
        assert_eq!(item.status, LegacyIsolationMilestoneStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_legacy_isolation_milestone("code-369", "name-983");
        if item.move_to(LegacyIsolationMilestoneStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyIsolationMilestoneStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_legacy_isolation_milestone("code-369", "name-983");
        item.status = LegacyIsolationMilestoneStatus::Expired;
        assert!(
            item.move_to(LegacyIsolationMilestoneStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
