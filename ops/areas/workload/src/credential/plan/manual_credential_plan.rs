use super::*;

use acme_kit::text;

/// Where a manual credential plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualCredentialPlanStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ManualCredentialPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ManualCredentialPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual credential plan as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialPlan {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub source: String,
    pub sequence: i64,
    pub status: ManualCredentialPlanStatus,
}

impl ManualCredentialPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualCredentialPlanStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualCredentialPlanStatus) -> bool {
        match self.status {
            ManualCredentialPlanStatus::Created => {
                match next {
                    ManualCredentialPlanStatus::Validated | ManualCredentialPlanStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualCredentialPlanStatus::Validated => {
                return next == ManualCredentialPlanStatus::Published || next == ManualCredentialPlanStatus::Retired;
            }
            ManualCredentialPlanStatus::Published => {
                return next == ManualCredentialPlanStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualCredentialPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual credential plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "manual credential plan", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.sequence as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualCredentialPlan {
        ManualCredentialPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            source: self.source.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a manual credential plan in status created.
pub fn new_manual_credential_plan(code: &str, label: &str) -> ManualCredentialPlan {
    ManualCredentialPlan {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        source: "".to_string(),
        sequence: 0,
        status: ManualCredentialPlanStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_manual_credential_plan("code-441", "label-307");
        assert_eq!(item.status, ManualCredentialPlanStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_manual_credential_plan("code-441", "label-307");
        if item.move_to(ManualCredentialPlanStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualCredentialPlanStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_manual_credential_plan("code-441", "label-307");
        item.status = ManualCredentialPlanStatus::Retired;
        assert!(
            item.move_to(ManualCredentialPlanStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
