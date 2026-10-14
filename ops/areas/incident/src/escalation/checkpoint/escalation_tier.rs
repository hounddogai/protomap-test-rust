use super::*;

use acme_kit::text;

/// Where a escalation tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationTierStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl EscalationTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for EscalationTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A escalation tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationTier {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub fingerprint: String,
    pub country: String,
    pub floor_value: f64,
    pub approved: bool,
    pub status: EscalationTierStatus,
}

impl EscalationTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationTierStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationTierStatus) -> bool {
        match self.status {
            EscalationTierStatus::Created => {
                return next == EscalationTierStatus::Validated || next == EscalationTierStatus::Retired;
            }
            EscalationTierStatus::Validated => {
                match next {
                    EscalationTierStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationTierStatus::Published => {
                return next == EscalationTierStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EscalationTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.minimum - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationTier {
        EscalationTier {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            fingerprint: self.fingerprint.clone(),
            country: self.country.clone(),
            floor_value: self.floor_value,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a escalation tier in status created.
pub fn new_escalation_tier(code: &str, title: &str) -> EscalationTier {
    EscalationTier {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        fingerprint: "".to_string(),
        country: "".to_string(),
        floor_value: 0.0,
        approved: false,
        status: EscalationTierStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_escalation_tier("code-763", "title-812");
        assert_eq!(item.status, EscalationTierStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_escalation_tier("code-763", "title-812");
        if item.move_to(EscalationTierStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationTierStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_escalation_tier("code-763", "title-812");
        item.status = EscalationTierStatus::Retired;
        assert!(
            item.move_to(EscalationTierStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
