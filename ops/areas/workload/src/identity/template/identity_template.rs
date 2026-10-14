use super::*;

use acme_kit::text;

/// Where a identity template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityTemplateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl IdentityTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for IdentityTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a identity template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityTemplate {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub cipher: String,
    pub unit: String,
    pub reviewer: String,
    pub status: IdentityTemplateStatus,
}

impl IdentityTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityTemplateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityTemplateStatus) -> bool {
        match self.status {
            IdentityTemplateStatus::Created => {
                match next {
                    IdentityTemplateStatus::Validated | IdentityTemplateStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityTemplateStatus::Validated => {
                match next {
                    IdentityTemplateStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityTemplateStatus::Published => {
                match next {
                    IdentityTemplateStatus::Retired => {
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

    pub fn move_to(&mut self, next: IdentityTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> IdentityTemplate {
        IdentityTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            cipher: self.cipher.clone(),
            unit: self.unit.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a identity template in status created.
pub fn new_identity_template(code: &str, label: &str) -> IdentityTemplate {
    IdentityTemplate {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        cipher: "".to_string(),
        unit: "".to_string(),
        reviewer: "".to_string(),
        status: IdentityTemplateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_identity_template("code-656", "label-573");
        assert_eq!(item.status, IdentityTemplateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_identity_template("code-656", "label-573");
        if item.move_to(IdentityTemplateStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityTemplateStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_identity_template("code-656", "label-573");
        item.status = IdentityTemplateStatus::Retired;
        assert!(
            item.move_to(IdentityTemplateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
