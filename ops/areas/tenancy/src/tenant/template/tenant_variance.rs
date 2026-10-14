use super::*;

use acme_kit::text;

/// Where a tenant variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantVarianceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TenantVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TenantVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tenant variance as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantVariance {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub issuer: String,
    pub port: i64,
    pub reviewer: String,
    pub approved: bool,
    pub host: String,
    pub country: String,
    pub status: TenantVarianceStatus,
}

impl TenantVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantVarianceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantVarianceStatus) -> bool {
        match self.status {
            TenantVarianceStatus::Proposed => {
                match next {
                    TenantVarianceStatus::Approved | TenantVarianceStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantVarianceStatus::Approved => {
                match next {
                    TenantVarianceStatus::Rejected | TenantVarianceStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantVarianceStatus::Rejected => {
                return next == TenantVarianceStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TenantVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tenant variance", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.port as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantVariance {
        TenantVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            issuer: self.issuer.clone(),
            port: self.port,
            reviewer: self.reviewer.clone(),
            approved: self.approved,
            host: self.host.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a tenant variance in status proposed.
pub fn new_tenant_variance(code: &str, label: &str) -> TenantVariance {
    TenantVariance {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        issuer: "".to_string(),
        port: 0,
        reviewer: "".to_string(),
        approved: false,
        host: "".to_string(),
        country: "".to_string(),
        status: TenantVarianceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_tenant_variance("code-37", "label-549");
        assert_eq!(item.status, TenantVarianceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_tenant_variance("code-37", "label-549");
        if item.move_to(TenantVarianceStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantVarianceStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_tenant_variance("code-37", "label-549");
        item.status = TenantVarianceStatus::Withdrawn;
        assert!(
            item.move_to(TenantVarianceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
