use super::*;

use acme_kit::text;

/// Where a egress plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressPlanStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl EgressPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for EgressPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A egress plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressPlan {
    pub code: String,
    pub title: String,
    pub floor_value: f64,
    pub created_at: i64,
    pub tags: Vec<String>,
    pub name: String,
    pub currency: String,
    pub contact_email: String,
    pub status: EgressPlanStatus,
}

impl EgressPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressPlanStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressPlanStatus) -> bool {
        match self.status {
            EgressPlanStatus::Proposed => {
                return next == EgressPlanStatus::Approved || next == EgressPlanStatus::Rejected;
            }
            EgressPlanStatus::Approved => {
                return next == EgressPlanStatus::Rejected || next == EgressPlanStatus::Withdrawn;
            }
            EgressPlanStatus::Rejected => {
                return next == EgressPlanStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.floor_value + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressPlan {
        EgressPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            floor_value: self.floor_value,
            created_at: self.created_at,
            tags: self.tags.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a egress plan in status proposed.
pub fn new_egress_plan(code: &str, title: &str) -> EgressPlan {
    EgressPlan {
        code: code.to_string(),
        title: title.to_string(),
        floor_value: 0.0,
        created_at: 0,
        tags: Vec::new(),
        name: "".to_string(),
        currency: "".to_string(),
        contact_email: "".to_string(),
        status: EgressPlanStatus::Proposed,
    }
}
