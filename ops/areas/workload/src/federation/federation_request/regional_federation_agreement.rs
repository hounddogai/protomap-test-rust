use super::*;

use acme_kit::text;

/// Where a regional federation agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalFederationAgreementStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionalFederationAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionalFederationAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional federation agreement as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFederationAgreement {
    pub code: String,
    pub title: String,
    pub category: String,
    pub namespace: String,
    pub fingerprint: String,
    pub locale: String,
    pub score: f64,
    pub status: RegionalFederationAgreementStatus,
}

impl RegionalFederationAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalFederationAgreementStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalFederationAgreementStatus) -> bool {
        match self.status {
            RegionalFederationAgreementStatus::Created => {
                return next == RegionalFederationAgreementStatus::Published;
            }
            RegionalFederationAgreementStatus::Validated => {
                return next == RegionalFederationAgreementStatus::Published;
            }
            RegionalFederationAgreementStatus::Published => {
                match next {
                    RegionalFederationAgreementStatus::Retired => {
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

    pub fn move_to(&mut self, next: RegionalFederationAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional federation agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional federation agreement", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> RegionalFederationAgreement {
        RegionalFederationAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            namespace: self.namespace.clone(),
            fingerprint: self.fingerprint.clone(),
            locale: self.locale.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a regional federation agreement in status created.
pub fn new_regional_federation_agreement(code: &str, title: &str) -> RegionalFederationAgreement {
    RegionalFederationAgreement {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        namespace: "".to_string(),
        fingerprint: "".to_string(),
        locale: "".to_string(),
        score: 0.0,
        status: RegionalFederationAgreementStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_regional_federation_agreement("code-577", "title-391");
        assert_eq!(item.status, RegionalFederationAgreementStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_regional_federation_agreement("code-577", "title-391");
        if item.move_to(RegionalFederationAgreementStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalFederationAgreementStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_regional_federation_agreement("code-577", "title-391");
        item.status = RegionalFederationAgreementStatus::Retired;
        assert!(
            item.move_to(RegionalFederationAgreementStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
