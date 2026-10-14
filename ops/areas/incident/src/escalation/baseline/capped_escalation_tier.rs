use super::*;

use acme_kit::text;

/// Where a capped escalation tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedEscalationTierStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CappedEscalationTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CappedEscalationTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped escalation tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedEscalationTier {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub principal: String,
    pub locale: String,
    pub unit: String,
    pub currency: String,
    pub ends_at: i64,
    pub segment: String,
    pub expires_at: i64,
    pub summary: String,
    pub status: CappedEscalationTierStatus,
}

impl CappedEscalationTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedEscalationTierStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedEscalationTierStatus) -> bool {
        match self.status {
            CappedEscalationTierStatus::Created => {
                match next {
                    CappedEscalationTierStatus::Validated | CappedEscalationTierStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedEscalationTierStatus::Validated => {
                return next == CappedEscalationTierStatus::Published;
            }
            CappedEscalationTierStatus::Published => {
                match next {
                    CappedEscalationTierStatus::Retired => {
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

    pub fn move_to(&mut self, next: CappedEscalationTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped escalation tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped escalation tier", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.expires_at as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedEscalationTier {
        CappedEscalationTier {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            principal: self.principal.clone(),
            locale: self.locale.clone(),
            unit: self.unit.clone(),
            currency: self.currency.clone(),
            ends_at: self.ends_at,
            segment: self.segment.clone(),
            expires_at: self.expires_at,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a capped escalation tier in status created.
pub fn new_capped_escalation_tier(code: &str, name: &str) -> CappedEscalationTier {
    CappedEscalationTier {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        principal: "".to_string(),
        locale: "".to_string(),
        unit: "".to_string(),
        currency: "".to_string(),
        ends_at: 0,
        segment: "".to_string(),
        expires_at: 0,
        summary: "".to_string(),
        status: CappedEscalationTierStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_capped_escalation_tier("code-816", "name-229");
        assert_eq!(item.status, CappedEscalationTierStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_capped_escalation_tier("code-816", "name-229");
        if item.move_to(CappedEscalationTierStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedEscalationTierStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_capped_escalation_tier("code-816", "name-229");
        item.status = CappedEscalationTierStatus::Retired;
        assert!(
            item.move_to(CappedEscalationTierStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
