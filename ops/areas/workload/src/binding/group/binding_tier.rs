use super::*;

use acme_kit::text;

/// Where a binding tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingTierStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BindingTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BindingTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A binding tier as the binding team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingTier {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub retries: i64,
    pub segment: String,
    pub revision: i64,
    pub issuer: String,
    pub status: BindingTierStatus,
}

impl BindingTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingTierStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingTierStatus) -> bool {
        match self.status {
            BindingTierStatus::Proposed => {
                match next {
                    BindingTierStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingTierStatus::Approved => {
                return next == BindingTierStatus::Rejected || next == BindingTierStatus::Withdrawn;
            }
            BindingTierStatus::Rejected => {
                match next {
                    BindingTierStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: BindingTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.revision as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingTier {
        BindingTier {
            code: self.code.clone(),
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            retries: self.retries,
            segment: self.segment.clone(),
            revision: self.revision,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a binding tier in status proposed.
pub fn new_binding_tier(code: &str, title: &str) -> BindingTier {
    BindingTier {
        code: code.to_string(),
        title: title.to_string(),
        tenant_id: "".to_string(),
        retries: 0,
        segment: "".to_string(),
        revision: 0,
        issuer: "".to_string(),
        status: BindingTierStatus::Proposed,
    }
}
