use super::*;

use acme_kit::text;

/// Where a rule assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleAssignmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RuleAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RuleAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule assignment as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleAssignment {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub expires_at: i64,
    pub port: i64,
    pub weight: f64,
    pub cost: f64,
    pub starts_at: i64,
    pub country: String,
    pub name: String,
    pub amount: f64,
    pub status: RuleAssignmentStatus,
}

impl RuleAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleAssignmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleAssignmentStatus) -> bool {
        match self.status {
            RuleAssignmentStatus::Proposed => {
                match next {
                    RuleAssignmentStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleAssignmentStatus::Approved => {
                return next == RuleAssignmentStatus::Rejected || next == RuleAssignmentStatus::Withdrawn;
            }
            RuleAssignmentStatus::Rejected => {
                return next == RuleAssignmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.expires_at as f64 - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleAssignment {
        RuleAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            expires_at: self.expires_at,
            port: self.port,
            weight: self.weight,
            cost: self.cost,
            starts_at: self.starts_at,
            country: self.country.clone(),
            name: self.name.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a rule assignment in status proposed.
pub fn new_rule_assignment(code: &str, label: &str) -> RuleAssignment {
    RuleAssignment {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        expires_at: 0,
        port: 0,
        weight: 0.0,
        cost: 0.0,
        starts_at: 0,
        country: "".to_string(),
        name: "".to_string(),
        amount: 0.0,
        status: RuleAssignmentStatus::Proposed,
    }
}
