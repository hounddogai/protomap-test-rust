use super::*;

use acme_kit::text;

/// Where a federation group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationGroupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FederationGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FederationGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a federation group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationGroup {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub budget: f64,
    pub weight: f64,
    pub scope: String,
    pub tags: Vec<String>,
    pub approved: bool,
    pub timezone: String,
    pub ceiling: f64,
    pub issuer: String,
    pub status: FederationGroupStatus,
}

impl FederationGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationGroupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationGroupStatus) -> bool {
        match self.status {
            FederationGroupStatus::Proposed => {
                match next {
                    FederationGroupStatus::Approved | FederationGroupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationGroupStatus::Approved => {
                return next == FederationGroupStatus::Withdrawn;
            }
            FederationGroupStatus::Rejected => {
                match next {
                    FederationGroupStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FederationGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_exposure(&self) -> f64 {
        (self.weight * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationGroup {
        FederationGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            budget: self.budget,
            weight: self.weight,
            scope: self.scope.clone(),
            tags: self.tags.clone(),
            approved: self.approved,
            timezone: self.timezone.clone(),
            ceiling: self.ceiling,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a federation group in status proposed.
pub fn new_federation_group(code: &str, name: &str) -> FederationGroup {
    FederationGroup {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        budget: 0.0,
        weight: 0.0,
        scope: "".to_string(),
        tags: Vec::new(),
        approved: false,
        timezone: "".to_string(),
        ceiling: 0.0,
        issuer: "".to_string(),
        status: FederationGroupStatus::Proposed,
    }
}
