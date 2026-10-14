use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a trust domain template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainTemplateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TrustDomainTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TrustDomainTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A trust domain template as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainTemplate {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub budget: f64,
    pub host: String,
    pub burst: i64,
    pub attributes: BTreeMap<String, String>,
    pub status: TrustDomainTemplateStatus,
}

impl TrustDomainTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainTemplateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainTemplateStatus) -> bool {
        match self.status {
            TrustDomainTemplateStatus::Created => {
                match next {
                    TrustDomainTemplateStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainTemplateStatus::Validated => {
                return next == TrustDomainTemplateStatus::Published;
            }
            TrustDomainTemplateStatus::Published => {
                match next {
                    TrustDomainTemplateStatus::Retired => {
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

    pub fn move_to(&mut self, next: TrustDomainTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "trust domain template", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn projected_total(&self) -> f64 {
        ((self.burst as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainTemplate {
        TrustDomainTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            budget: self.budget,
            host: self.host.clone(),
            burst: self.burst,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a trust domain template in status created.
pub fn new_trust_domain_template(code: &str, label: &str) -> TrustDomainTemplate {
    TrustDomainTemplate {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        budget: 0.0,
        host: "".to_string(),
        burst: 0,
        attributes: BTreeMap::new(),
        status: TrustDomainTemplateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_trust_domain_template("code-668", "label-478");
        assert_eq!(item.status, TrustDomainTemplateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_trust_domain_template("code-668", "label-478");
        if item.move_to(TrustDomainTemplateStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainTemplateStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_trust_domain_template("code-668", "label-478");
        item.status = TrustDomainTemplateStatus::Retired;
        assert!(
            item.move_to(TrustDomainTemplateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
