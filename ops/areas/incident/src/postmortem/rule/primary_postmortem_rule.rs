use super::*;

use acme_kit::text;

/// Where a primary postmortem rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryPostmortemRuleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PrimaryPostmortemRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PrimaryPostmortemRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary postmortem rule as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPostmortemRule {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub fingerprint: String,
    pub duration_minutes: i64,
    pub description: String,
    pub approved: bool,
    pub issuer: String,
    pub namespace: String,
    pub status: PrimaryPostmortemRuleStatus,
}

impl PrimaryPostmortemRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryPostmortemRuleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryPostmortemRuleStatus) -> bool {
        match self.status {
            PrimaryPostmortemRuleStatus::Created => {
                match next {
                    PrimaryPostmortemRuleStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryPostmortemRuleStatus::Validated => {
                return next == PrimaryPostmortemRuleStatus::Published || next == PrimaryPostmortemRuleStatus::Retired;
            }
            PrimaryPostmortemRuleStatus::Published => {
                match next {
                    PrimaryPostmortemRuleStatus::Retired => {
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

    pub fn move_to(&mut self, next: PrimaryPostmortemRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary postmortem rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.duration_minutes as f64
    }

    pub fn copy(&self) -> PrimaryPostmortemRule {
        PrimaryPostmortemRule {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            fingerprint: self.fingerprint.clone(),
            duration_minutes: self.duration_minutes,
            description: self.description.clone(),
            approved: self.approved,
            issuer: self.issuer.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a primary postmortem rule in status created.
pub fn new_primary_postmortem_rule(code: &str, title: &str) -> PrimaryPostmortemRule {
    PrimaryPostmortemRule {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        fingerprint: "".to_string(),
        duration_minutes: 0,
        description: "".to_string(),
        approved: false,
        issuer: "".to_string(),
        namespace: "".to_string(),
        status: PrimaryPostmortemRuleStatus::Created,
    }
}
