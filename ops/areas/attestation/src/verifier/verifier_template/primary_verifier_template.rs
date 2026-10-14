use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a primary verifier template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryVerifierTemplateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryVerifierTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryVerifierTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary verifier template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierTemplate {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub ceiling: f64,
    pub attributes: BTreeMap<String, String>,
    pub rate: f64,
    pub status: PrimaryVerifierTemplateStatus,
}

impl PrimaryVerifierTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryVerifierTemplateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryVerifierTemplateStatus) -> bool {
        match self.status {
            PrimaryVerifierTemplateStatus::Requested => {
                match next {
                    PrimaryVerifierTemplateStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryVerifierTemplateStatus::Granted => {
                match next {
                    PrimaryVerifierTemplateStatus::Revoked | PrimaryVerifierTemplateStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryVerifierTemplateStatus::Revoked => {
                match next {
                    PrimaryVerifierTemplateStatus::Expired => {
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

    pub fn move_to(&mut self, next: PrimaryVerifierTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary verifier template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary verifier template", self.code)
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
        (self.rate * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryVerifierTemplate {
        PrimaryVerifierTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            ceiling: self.ceiling,
            attributes: self.attributes.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a primary verifier template in status requested.
pub fn new_primary_verifier_template(code: &str, name: &str) -> PrimaryVerifierTemplate {
    PrimaryVerifierTemplate {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        ceiling: 0.0,
        attributes: BTreeMap::new(),
        rate: 0.0,
        status: PrimaryVerifierTemplateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_primary_verifier_template("code-927", "name-556");
        assert_eq!(item.status, PrimaryVerifierTemplateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_primary_verifier_template("code-927", "name-556");
        if item.move_to(PrimaryVerifierTemplateStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryVerifierTemplateStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_primary_verifier_template("code-927", "name-556");
        item.status = PrimaryVerifierTemplateStatus::Expired;
        assert!(
            item.move_to(PrimaryVerifierTemplateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
