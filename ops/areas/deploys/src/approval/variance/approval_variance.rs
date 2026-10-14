use super::*;

use acme_kit::text;

/// Where a approval variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalVarianceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ApprovalVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ApprovalVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval variance as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalVariance {
    pub code: String,
    pub label: String,
    pub budget: f64,
    pub name: String,
    pub ratio: f64,
    pub version: i64,
    pub cluster: String,
    pub region: String,
    pub quota_units: i64,
    pub weight: f64,
    pub status: ApprovalVarianceStatus,
}

impl ApprovalVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalVarianceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalVarianceStatus) -> bool {
        match self.status {
            ApprovalVarianceStatus::Requested => {
                return next == ApprovalVarianceStatus::Revoked || next == ApprovalVarianceStatus::Expired;
            }
            ApprovalVarianceStatus::Granted => {
                return next == ApprovalVarianceStatus::Revoked || next == ApprovalVarianceStatus::Expired;
            }
            ApprovalVarianceStatus::Revoked => {
                return next == ApprovalVarianceStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ApprovalVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.quota_units as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalVariance {
        ApprovalVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            budget: self.budget,
            name: self.name.clone(),
            ratio: self.ratio,
            version: self.version,
            cluster: self.cluster.clone(),
            region: self.region.clone(),
            quota_units: self.quota_units,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a approval variance in status requested.
pub fn new_approval_variance(code: &str, label: &str) -> ApprovalVariance {
    ApprovalVariance {
        code: code.to_string(),
        label: label.to_string(),
        budget: 0.0,
        name: "".to_string(),
        ratio: 0.0,
        version: 0,
        cluster: "".to_string(),
        region: "".to_string(),
        quota_units: 0,
        weight: 0.0,
        status: ApprovalVarianceStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_approval_variance("code-203", "label-716");
        assert_eq!(item.status, ApprovalVarianceStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_approval_variance("code-203", "label-716");
        if item.move_to(ApprovalVarianceStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalVarianceStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_approval_variance("code-203", "label-716");
        item.status = ApprovalVarianceStatus::Expired;
        assert!(
            item.move_to(ApprovalVarianceStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
