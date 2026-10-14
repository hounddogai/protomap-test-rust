use super::*;

use acme_kit::text;

/// Where a primary overage quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryOverageQuotaStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimaryOverageQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimaryOverageQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary overage quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageQuota {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub source: String,
    pub floor_value: f64,
    pub external_ref: String,
    pub score: f64,
    pub audience: String,
    pub timezone: String,
    pub cluster: String,
    pub status: PrimaryOverageQuotaStatus,
}

impl PrimaryOverageQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryOverageQuotaStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryOverageQuotaStatus) -> bool {
        match self.status {
            PrimaryOverageQuotaStatus::Proposed => {
                match next {
                    PrimaryOverageQuotaStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryOverageQuotaStatus::Approved => {
                return next == PrimaryOverageQuotaStatus::Rejected || next == PrimaryOverageQuotaStatus::Withdrawn;
            }
            PrimaryOverageQuotaStatus::Rejected => {
                match next {
                    PrimaryOverageQuotaStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PrimaryOverageQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary overage quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.score * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryOverageQuota {
        PrimaryOverageQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            source: self.source.clone(),
            floor_value: self.floor_value,
            external_ref: self.external_ref.clone(),
            score: self.score,
            audience: self.audience.clone(),
            timezone: self.timezone.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a primary overage quota in status proposed.
pub fn new_primary_overage_quota(code: &str, label: &str) -> PrimaryOverageQuota {
    PrimaryOverageQuota {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        source: "".to_string(),
        floor_value: 0.0,
        external_ref: "".to_string(),
        score: 0.0,
        audience: "".to_string(),
        timezone: "".to_string(),
        cluster: "".to_string(),
        status: PrimaryOverageQuotaStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_primary_overage_quota("code-447", "label-345");
        assert_eq!(item.status, PrimaryOverageQuotaStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_primary_overage_quota("code-447", "label-345");
        if item.move_to(PrimaryOverageQuotaStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryOverageQuotaStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_primary_overage_quota("code-447", "label-345");
        item.status = PrimaryOverageQuotaStatus::Withdrawn;
        assert!(
            item.move_to(PrimaryOverageQuotaStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
