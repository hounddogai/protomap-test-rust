use super::*;

use acme_kit::text;

/// Where a capped peering budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedPeeringBudgetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CappedPeeringBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CappedPeeringBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped peering budget as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPeeringBudget {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub visible: bool,
    pub port: i64,
    pub expires_at: i64,
    pub currency: String,
    pub version: i64,
    pub namespace: String,
    pub rotation_days: i64,
    pub host: String,
    pub status: CappedPeeringBudgetStatus,
}

impl CappedPeeringBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedPeeringBudgetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedPeeringBudgetStatus) -> bool {
        match self.status {
            CappedPeeringBudgetStatus::Created => {
                return next == CappedPeeringBudgetStatus::Validated || next == CappedPeeringBudgetStatus::Published;
            }
            CappedPeeringBudgetStatus::Validated => {
                match next {
                    CappedPeeringBudgetStatus::Published | CappedPeeringBudgetStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedPeeringBudgetStatus::Published => {
                match next {
                    CappedPeeringBudgetStatus::Retired => {
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

    pub fn move_to(&mut self, next: CappedPeeringBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped peering budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "capped peering budget", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.version as f64 - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedPeeringBudget {
        CappedPeeringBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            visible: self.visible,
            port: self.port,
            expires_at: self.expires_at,
            currency: self.currency.clone(),
            version: self.version,
            namespace: self.namespace.clone(),
            rotation_days: self.rotation_days,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a capped peering budget in status created.
pub fn new_capped_peering_budget(code: &str, label: &str) -> CappedPeeringBudget {
    CappedPeeringBudget {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        visible: false,
        port: 0,
        expires_at: 0,
        currency: "".to_string(),
        version: 0,
        namespace: "".to_string(),
        rotation_days: 0,
        host: "".to_string(),
        status: CappedPeeringBudgetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_capped_peering_budget("code-839", "label-579");
        assert_eq!(item.status, CappedPeeringBudgetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_capped_peering_budget("code-839", "label-579");
        if item.move_to(CappedPeeringBudgetStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedPeeringBudgetStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_capped_peering_budget("code-839", "label-579");
        item.status = CappedPeeringBudgetStatus::Retired;
        assert!(
            item.move_to(CappedPeeringBudgetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
