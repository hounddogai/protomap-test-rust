use super::*;

use acme_kit::text;

/// Where a regional peering budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalPeeringBudgetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalPeeringBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalPeeringBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional peering budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPeeringBudget {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub minimum: f64,
    pub version: i64,
    pub starts_at: i64,
    pub contact_email: String,
    pub cipher: String,
    pub segment: String,
    pub attempts: i64,
    pub status: RegionalPeeringBudgetStatus,
}

impl RegionalPeeringBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalPeeringBudgetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalPeeringBudgetStatus) -> bool {
        match self.status {
            RegionalPeeringBudgetStatus::Proposed => {
                return next == RegionalPeeringBudgetStatus::Withdrawn;
            }
            RegionalPeeringBudgetStatus::Approved => {
                match next {
                    RegionalPeeringBudgetStatus::Rejected | RegionalPeeringBudgetStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalPeeringBudgetStatus::Rejected => {
                return next == RegionalPeeringBudgetStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalPeeringBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional peering budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.attempts as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalPeeringBudget {
        RegionalPeeringBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            minimum: self.minimum,
            version: self.version,
            starts_at: self.starts_at,
            contact_email: self.contact_email.clone(),
            cipher: self.cipher.clone(),
            segment: self.segment.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a regional peering budget in status proposed.
pub fn new_regional_peering_budget(code: &str, title: &str) -> RegionalPeeringBudget {
    RegionalPeeringBudget {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        minimum: 0.0,
        version: 0,
        starts_at: 0,
        contact_email: "".to_string(),
        cipher: "".to_string(),
        segment: "".to_string(),
        attempts: 0,
        status: RegionalPeeringBudgetStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_regional_peering_budget("code-492", "title-294");
        assert_eq!(item.status, RegionalPeeringBudgetStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_regional_peering_budget("code-492", "title-294");
        if item.move_to(RegionalPeeringBudgetStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalPeeringBudgetStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_regional_peering_budget("code-492", "title-294");
        item.status = RegionalPeeringBudgetStatus::Withdrawn;
        assert!(
            item.move_to(RegionalPeeringBudgetStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
