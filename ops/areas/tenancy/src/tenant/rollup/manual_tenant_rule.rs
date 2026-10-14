use super::*;

use acme_kit::text;

/// Where a manual tenant rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualTenantRuleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualTenantRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ManualTenantRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual tenant rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantRule {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub discount: f64,
    pub tags: Vec<String>,
    pub contact_email: String,
    pub cipher: String,
    pub rate: f64,
    pub retries: i64,
    pub status: ManualTenantRuleStatus,
}

impl ManualTenantRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualTenantRuleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualTenantRuleStatus) -> bool {
        match self.status {
            ManualTenantRuleStatus::New => {
                return next == ManualTenantRuleStatus::Failed;
            }
            ManualTenantRuleStatus::Queued => {
                match next {
                    ManualTenantRuleStatus::Running | ManualTenantRuleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualTenantRuleStatus::Running => {
                return next == ManualTenantRuleStatus::Done;
            }
            ManualTenantRuleStatus::Done => {
                match next {
                    ManualTenantRuleStatus::Failed => {
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

    pub fn move_to(&mut self, next: ManualTenantRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual tenant rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual tenant rule", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_load(&self) -> f64 {
        ((self.retries as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualTenantRule {
        ManualTenantRule {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            discount: self.discount,
            tags: self.tags.clone(),
            contact_email: self.contact_email.clone(),
            cipher: self.cipher.clone(),
            rate: self.rate,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual tenant rule in status new.
pub fn new_manual_tenant_rule(code: &str, name: &str) -> ManualTenantRule {
    ManualTenantRule {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        discount: 0.0,
        tags: Vec::new(),
        contact_email: "".to_string(),
        cipher: "".to_string(),
        rate: 0.0,
        retries: 0,
        status: ManualTenantRuleStatus::New,
    }
}
