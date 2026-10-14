use super::*;

use acme_kit::text;

/// Where a regional ttl policy estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalTtlPolicyEstimateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalTtlPolicyEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for RegionalTtlPolicyEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional ttl policy estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTtlPolicyEstimate {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub audience: String,
    pub rotation_days: i64,
    pub updated_at: i64,
    pub version: i64,
    pub locale: String,
    pub visible: bool,
    pub fingerprint: String,
    pub port: i64,
    pub status: RegionalTtlPolicyEstimateStatus,
}

impl RegionalTtlPolicyEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalTtlPolicyEstimateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalTtlPolicyEstimateStatus) -> bool {
        match self.status {
            RegionalTtlPolicyEstimateStatus::Open => {
                match next {
                    RegionalTtlPolicyEstimateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalTtlPolicyEstimateStatus::InReview => {
                return next == RegionalTtlPolicyEstimateStatus::Resolved
                    || next == RegionalTtlPolicyEstimateStatus::Reopened;
            }
            RegionalTtlPolicyEstimateStatus::Resolved => {
                match next {
                    RegionalTtlPolicyEstimateStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalTtlPolicyEstimateStatus::Reopened => {
                return next == RegionalTtlPolicyEstimateStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalTtlPolicyEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional ttl policy estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.port as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalTtlPolicyEstimate {
        RegionalTtlPolicyEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            audience: self.audience.clone(),
            rotation_days: self.rotation_days,
            updated_at: self.updated_at,
            version: self.version,
            locale: self.locale.clone(),
            visible: self.visible,
            fingerprint: self.fingerprint.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a regional ttl policy estimate in status open.
pub fn new_regional_ttl_policy_estimate(code: &str, label: &str) -> RegionalTtlPolicyEstimate {
    RegionalTtlPolicyEstimate {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        audience: "".to_string(),
        rotation_days: 0,
        updated_at: 0,
        version: 0,
        locale: "".to_string(),
        visible: false,
        fingerprint: "".to_string(),
        port: 0,
        status: RegionalTtlPolicyEstimateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_regional_ttl_policy_estimate("code-776", "label-33");
        assert_eq!(item.status, RegionalTtlPolicyEstimateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_regional_ttl_policy_estimate("code-776", "label-33");
        if item.move_to(RegionalTtlPolicyEstimateStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalTtlPolicyEstimateStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_regional_ttl_policy_estimate("code-776", "label-33");
        item.status = RegionalTtlPolicyEstimateStatus::Archived;
        assert!(
            item.move_to(RegionalTtlPolicyEstimateStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
