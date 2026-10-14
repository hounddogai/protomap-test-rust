use super::*;

use acme_kit::text;

/// Where a egress override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressOverrideStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl EgressOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for EgressOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a egress override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressOverride {
    pub code: String,
    pub label: String,
    pub quota_units: i64,
    pub updated_at: i64,
    pub tenant_id: String,
    pub weight: f64,
    pub source: String,
    pub summary: String,
    pub port: i64,
    pub category: String,
    pub status: EgressOverrideStatus,
}

impl EgressOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressOverrideStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressOverrideStatus) -> bool {
        match self.status {
            EgressOverrideStatus::Proposed => {
                return next == EgressOverrideStatus::Rejected;
            }
            EgressOverrideStatus::Approved => {
                return next == EgressOverrideStatus::Rejected || next == EgressOverrideStatus::Withdrawn;
            }
            EgressOverrideStatus::Rejected => {
                match next {
                    EgressOverrideStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: EgressOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress override {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.port as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressOverride {
        EgressOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            updated_at: self.updated_at,
            tenant_id: self.tenant_id.clone(),
            weight: self.weight,
            source: self.source.clone(),
            summary: self.summary.clone(),
            port: self.port,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a egress override in status proposed.
pub fn new_egress_override(code: &str, label: &str) -> EgressOverride {
    EgressOverride {
        code: code.to_string(),
        label: label.to_string(),
        quota_units: 0,
        updated_at: 0,
        tenant_id: "".to_string(),
        weight: 0.0,
        source: "".to_string(),
        summary: "".to_string(),
        port: 0,
        category: "".to_string(),
        status: EgressOverrideStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_egress_override("code-755", "label-765");
        assert_eq!(item.status, EgressOverrideStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_egress_override("code-755", "label-765");
        if item.move_to(EgressOverrideStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressOverrideStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_egress_override("code-755", "label-765");
        item.status = EgressOverrideStatus::Withdrawn;
        assert!(
            item.move_to(EgressOverrideStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
