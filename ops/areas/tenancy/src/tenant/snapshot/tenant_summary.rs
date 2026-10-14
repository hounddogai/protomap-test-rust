use super::*;

use acme_kit::text;

/// Where a tenant summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantSummaryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TenantSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TenantSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tenant summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantSummary {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub reviewer: String,
    pub tenant_id: String,
    pub approved: bool,
    pub name: String,
    pub score: f64,
    pub status: TenantSummaryStatus,
}

impl TenantSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantSummaryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantSummaryStatus) -> bool {
        match self.status {
            TenantSummaryStatus::Proposed => {
                match next {
                    TenantSummaryStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantSummaryStatus::Approved => {
                match next {
                    TenantSummaryStatus::Rejected | TenantSummaryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantSummaryStatus::Rejected => {
                match next {
                    TenantSummaryStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TenantSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> TenantSummary {
        TenantSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            reviewer: self.reviewer.clone(),
            tenant_id: self.tenant_id.clone(),
            approved: self.approved,
            name: self.name.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a tenant summary in status proposed.
pub fn new_tenant_summary(code: &str, label: &str) -> TenantSummary {
    TenantSummary {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        reviewer: "".to_string(),
        tenant_id: "".to_string(),
        approved: false,
        name: "".to_string(),
        score: 0.0,
        status: TenantSummaryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_tenant_summary("code-911", "label-757");
        assert_eq!(item.status, TenantSummaryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_tenant_summary("code-911", "label-757");
        if item.move_to(TenantSummaryStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantSummaryStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_tenant_summary("code-911", "label-757");
        item.status = TenantSummaryStatus::Withdrawn;
        assert!(
            item.move_to(TenantSummaryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
