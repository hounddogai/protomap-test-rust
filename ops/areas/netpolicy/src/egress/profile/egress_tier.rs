use super::*;

use acme_kit::text;

/// Where a egress tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressTierStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl EgressTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for EgressTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One egress tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressTier {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub starts_at: i64,
    pub quantity: i64,
    pub region: String,
    pub contact_email: String,
    pub margin: f64,
    pub amount: f64,
    pub status: EgressTierStatus,
}

impl EgressTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressTierStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressTierStatus) -> bool {
        match self.status {
            EgressTierStatus::Created => {
                return next == EgressTierStatus::Validated;
            }
            EgressTierStatus::Validated => {
                return next == EgressTierStatus::Retired;
            }
            EgressTierStatus::Published => {
                return next == EgressTierStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.margin + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressTier {
        EgressTier {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            starts_at: self.starts_at,
            quantity: self.quantity,
            region: self.region.clone(),
            contact_email: self.contact_email.clone(),
            margin: self.margin,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a egress tier in status created.
pub fn new_egress_tier(code: &str, name: &str) -> EgressTier {
    EgressTier {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        starts_at: 0,
        quantity: 0,
        region: "".to_string(),
        contact_email: "".to_string(),
        margin: 0.0,
        amount: 0.0,
        status: EgressTierStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_egress_tier("code-465", "name-130");
        assert_eq!(item.status, EgressTierStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_egress_tier("code-465", "name-130");
        if item.move_to(EgressTierStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressTierStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_egress_tier("code-465", "name-130");
        item.status = EgressTierStatus::Retired;
        assert!(
            item.move_to(EgressTierStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
