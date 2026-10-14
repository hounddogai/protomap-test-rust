use super::*;

use acme_kit::text;

/// Where a rule manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleManifestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RuleManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RuleManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rule manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleManifest {
    pub code: String,
    pub title: String,
    pub principal: String,
    pub contact_email: String,
    pub budget: f64,
    pub cipher: String,
    pub threshold: f64,
    pub issuer: String,
    pub tenant_id: String,
    pub discount: f64,
    pub key_id: String,
    pub status: RuleManifestStatus,
}

impl RuleManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleManifestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleManifestStatus) -> bool {
        match self.status {
            RuleManifestStatus::Created => {
                match next {
                    RuleManifestStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleManifestStatus::Validated => {
                return next == RuleManifestStatus::Published || next == RuleManifestStatus::Retired;
            }
            RuleManifestStatus::Published => {
                match next {
                    RuleManifestStatus::Retired => {
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

    pub fn move_to(&mut self, next: RuleManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.threshold * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleManifest {
        RuleManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            principal: self.principal.clone(),
            contact_email: self.contact_email.clone(),
            budget: self.budget,
            cipher: self.cipher.clone(),
            threshold: self.threshold,
            issuer: self.issuer.clone(),
            tenant_id: self.tenant_id.clone(),
            discount: self.discount,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rule manifest in status created.
pub fn new_rule_manifest(code: &str, title: &str) -> RuleManifest {
    RuleManifest {
        code: code.to_string(),
        title: title.to_string(),
        principal: "".to_string(),
        contact_email: "".to_string(),
        budget: 0.0,
        cipher: "".to_string(),
        threshold: 0.0,
        issuer: "".to_string(),
        tenant_id: "".to_string(),
        discount: 0.0,
        key_id: "".to_string(),
        status: RuleManifestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_rule_manifest("code-632", "title-925");
        assert_eq!(item.status, RuleManifestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_rule_manifest("code-632", "title-925");
        if item.move_to(RuleManifestStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleManifestStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_rule_manifest("code-632", "title-925");
        item.status = RuleManifestStatus::Retired;
        assert!(
            item.move_to(RuleManifestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
