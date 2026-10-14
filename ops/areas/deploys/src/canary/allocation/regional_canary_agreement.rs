use super::*;

use acme_kit::text;

/// Where a regional canary agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalCanaryAgreementStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionalCanaryAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionalCanaryAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional canary agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanaryAgreement {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub namespace: String,
    pub currency: String,
    pub source: String,
    pub amount: f64,
    pub status: RegionalCanaryAgreementStatus,
}

impl RegionalCanaryAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalCanaryAgreementStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalCanaryAgreementStatus) -> bool {
        match self.status {
            RegionalCanaryAgreementStatus::Created => {
                return next == RegionalCanaryAgreementStatus::Validated;
            }
            RegionalCanaryAgreementStatus::Validated => {
                match next {
                    RegionalCanaryAgreementStatus::Published | RegionalCanaryAgreementStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalCanaryAgreementStatus::Published => {
                return next == RegionalCanaryAgreementStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalCanaryAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional canary agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional canary agreement", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> RegionalCanaryAgreement {
        RegionalCanaryAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            namespace: self.namespace.clone(),
            currency: self.currency.clone(),
            source: self.source.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a regional canary agreement in status created.
pub fn new_regional_canary_agreement(code: &str, name: &str) -> RegionalCanaryAgreement {
    RegionalCanaryAgreement {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        namespace: "".to_string(),
        currency: "".to_string(),
        source: "".to_string(),
        amount: 0.0,
        status: RegionalCanaryAgreementStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_regional_canary_agreement("code-921", "name-179");
        assert_eq!(item.status, RegionalCanaryAgreementStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_regional_canary_agreement("code-921", "name-179");
        if item.move_to(RegionalCanaryAgreementStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalCanaryAgreementStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_regional_canary_agreement("code-921", "name-179");
        item.status = RegionalCanaryAgreementStatus::Retired;
        assert!(
            item.move_to(RegionalCanaryAgreementStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
