use super::*;

use acme_kit::text;

/// Where a manual tenant target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualTenantTargetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualTenantTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualTenantTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual tenant target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantTarget {
    pub code: String,
    pub title: String,
    pub country: String,
    pub window_days: i64,
    pub floor_value: f64,
    pub weight: f64,
    pub retries: i64,
    pub revision: i64,
    pub external_ref: String,
    pub rate: f64,
    pub status: ManualTenantTargetStatus,
}

impl ManualTenantTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualTenantTargetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualTenantTargetStatus) -> bool {
        match self.status {
            ManualTenantTargetStatus::Requested => {
                match next {
                    ManualTenantTargetStatus::Granted | ManualTenantTargetStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualTenantTargetStatus::Granted => {
                match next {
                    ManualTenantTargetStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualTenantTargetStatus::Revoked => {
                return next == ManualTenantTargetStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualTenantTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual tenant target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual tenant target", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.floor_value - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualTenantTarget {
        ManualTenantTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            window_days: self.window_days,
            floor_value: self.floor_value,
            weight: self.weight,
            retries: self.retries,
            revision: self.revision,
            external_ref: self.external_ref.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a manual tenant target in status requested.
pub fn new_manual_tenant_target(code: &str, title: &str) -> ManualTenantTarget {
    ManualTenantTarget {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        window_days: 0,
        floor_value: 0.0,
        weight: 0.0,
        retries: 0,
        revision: 0,
        external_ref: "".to_string(),
        rate: 0.0,
        status: ManualTenantTargetStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_tenant_target("code-831", "title-183");
        assert_eq!(item.status, ManualTenantTargetStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_manual_tenant_target("code-831", "title-183");
        if item.move_to(ManualTenantTargetStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualTenantTargetStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_tenant_target("code-831", "title-183");
        item.status = ManualTenantTargetStatus::Expired;
        assert!(
            item.move_to(ManualTenantTargetStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
