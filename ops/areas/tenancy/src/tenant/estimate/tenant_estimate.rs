use super::*;

use acme_kit::text;

/// Where a tenant estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantEstimateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TenantEstimateStatus {
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

impl std::fmt::Display for TenantEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tenant estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantEstimate {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub cluster: String,
    pub scope: String,
    pub checksum: String,
    pub channel: String,
    pub amount: f64,
    pub status: TenantEstimateStatus,
}

impl TenantEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantEstimateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantEstimateStatus) -> bool {
        match self.status {
            TenantEstimateStatus::Open => {
                match next {
                    TenantEstimateStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantEstimateStatus::InReview => {
                match next {
                    TenantEstimateStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantEstimateStatus::Resolved => {
                return next == TenantEstimateStatus::Archived;
            }
            TenantEstimateStatus::Reopened => {
                return next == TenantEstimateStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TenantEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> TenantEstimate {
        TenantEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            cluster: self.cluster.clone(),
            scope: self.scope.clone(),
            checksum: self.checksum.clone(),
            channel: self.channel.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a tenant estimate in status open.
pub fn new_tenant_estimate(code: &str, name: &str) -> TenantEstimate {
    TenantEstimate {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        cluster: "".to_string(),
        scope: "".to_string(),
        checksum: "".to_string(),
        channel: "".to_string(),
        amount: 0.0,
        status: TenantEstimateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tenant_estimate("code-986", "name-706");
        assert_eq!(item.status, TenantEstimateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_tenant_estimate("code-986", "name-706");
        if item.move_to(TenantEstimateStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantEstimateStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tenant_estimate("code-986", "name-706");
        item.status = TenantEstimateStatus::Archived;
        assert!(
            item.move_to(TenantEstimateStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
