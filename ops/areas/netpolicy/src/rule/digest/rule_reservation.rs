use super::*;

use acme_kit::text;

/// Where a rule reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleReservationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RuleReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RuleReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule reservation as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleReservation {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub host: String,
    pub tenant_id: String,
    pub amount: f64,
    pub status: RuleReservationStatus,
}

impl RuleReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleReservationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleReservationStatus) -> bool {
        match self.status {
            RuleReservationStatus::Proposed => {
                return next == RuleReservationStatus::Rejected || next == RuleReservationStatus::Withdrawn;
            }
            RuleReservationStatus::Approved => {
                match next {
                    RuleReservationStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleReservationStatus::Rejected => {
                return next == RuleReservationStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> RuleReservation {
        RuleReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            host: self.host.clone(),
            tenant_id: self.tenant_id.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a rule reservation in status proposed.
pub fn new_rule_reservation(code: &str, name: &str) -> RuleReservation {
    RuleReservation {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        host: "".to_string(),
        tenant_id: "".to_string(),
        amount: 0.0,
        status: RuleReservationStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rule_reservation("code-39", "name-842");
        assert_eq!(item.status, RuleReservationStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_rule_reservation("code-39", "name-842");
        if item.move_to(RuleReservationStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleReservationStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rule_reservation("code-39", "name-842");
        item.status = RuleReservationStatus::Withdrawn;
        assert!(
            item.move_to(RuleReservationStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
