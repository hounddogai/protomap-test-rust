use super::*;

use acme_kit::text;

/// Where a primary credential plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryCredentialPlanStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PrimaryCredentialPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for PrimaryCredentialPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary credential plan as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCredentialPlan {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub quota_units: i64,
    pub discount: f64,
    pub status: PrimaryCredentialPlanStatus,
}

impl PrimaryCredentialPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryCredentialPlanStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryCredentialPlanStatus) -> bool {
        match self.status {
            PrimaryCredentialPlanStatus::Draft => {
                match next {
                    PrimaryCredentialPlanStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryCredentialPlanStatus::Pending => {
                match next {
                    PrimaryCredentialPlanStatus::Active | PrimaryCredentialPlanStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryCredentialPlanStatus::Active => {
                match next {
                    PrimaryCredentialPlanStatus::Suspended | PrimaryCredentialPlanStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryCredentialPlanStatus::Suspended => {
                match next {
                    PrimaryCredentialPlanStatus::Closed => {
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

    pub fn move_to(&mut self, next: PrimaryCredentialPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary credential plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.quota_units as f64 + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryCredentialPlan {
        PrimaryCredentialPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            checksum: self.checksum.clone(),
            quota_units: self.quota_units,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary credential plan in status draft.
pub fn new_primary_credential_plan(code: &str, label: &str) -> PrimaryCredentialPlan {
    PrimaryCredentialPlan {
        code: code.to_string(),
        label: label.to_string(),
        checksum: "".to_string(),
        quota_units: 0,
        discount: 0.0,
        status: PrimaryCredentialPlanStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_primary_credential_plan("code-628", "label-913");
        assert_eq!(item.status, PrimaryCredentialPlanStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_primary_credential_plan("code-628", "label-913");
        if item.move_to(PrimaryCredentialPlanStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryCredentialPlanStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_primary_credential_plan("code-628", "label-913");
        item.status = PrimaryCredentialPlanStatus::Closed;
        assert!(
            item.move_to(PrimaryCredentialPlanStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
