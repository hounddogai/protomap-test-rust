use super::*;

use acme_kit::text;

/// Where a egress rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressRollupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl EgressRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for EgressRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a egress rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressRollup {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub active: bool,
    pub discount: f64,
    pub ttl_seconds: i64,
    pub locale: String,
    pub maximum: f64,
    pub currency: String,
    pub status: EgressRollupStatus,
}

impl EgressRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressRollupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressRollupStatus) -> bool {
        match self.status {
            EgressRollupStatus::Created => {
                match next {
                    EgressRollupStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressRollupStatus::Validated => {
                match next {
                    EgressRollupStatus::Published | EgressRollupStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressRollupStatus::Published => {
                match next {
                    EgressRollupStatus::Retired => {
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

    pub fn move_to(&mut self, next: EgressRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.cost + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressRollup {
        EgressRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            active: self.active,
            discount: self.discount,
            ttl_seconds: self.ttl_seconds,
            locale: self.locale.clone(),
            maximum: self.maximum,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a egress rollup in status created.
pub fn new_egress_rollup(code: &str, name: &str) -> EgressRollup {
    EgressRollup {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        active: false,
        discount: 0.0,
        ttl_seconds: 0,
        locale: "".to_string(),
        maximum: 0.0,
        currency: "".to_string(),
        status: EgressRollupStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_egress_rollup("code-397", "name-478");
        assert_eq!(item.status, EgressRollupStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_egress_rollup("code-397", "name-478");
        if item.move_to(EgressRollupStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressRollupStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_egress_rollup("code-397", "name-478");
        item.status = EgressRollupStatus::Retired;
        assert!(
            item.move_to(EgressRollupStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
