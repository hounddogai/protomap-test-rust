use super::*;

use acme_kit::text;

/// Where a legacy binding template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyBindingTemplateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacyBindingTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacyBindingTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy binding template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingTemplate {
    pub code: String,
    pub title: String,
    pub visible: bool,
    pub quota_units: i64,
    pub cipher: String,
    pub issuer: String,
    pub status: LegacyBindingTemplateStatus,
}

impl LegacyBindingTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyBindingTemplateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyBindingTemplateStatus) -> bool {
        match self.status {
            LegacyBindingTemplateStatus::Created => {
                match next {
                    LegacyBindingTemplateStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyBindingTemplateStatus::Validated => {
                match next {
                    LegacyBindingTemplateStatus::Published | LegacyBindingTemplateStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyBindingTemplateStatus::Published => {
                return next == LegacyBindingTemplateStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyBindingTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy binding template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.quota_units as f64
    }

    pub fn copy(&self) -> LegacyBindingTemplate {
        LegacyBindingTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            visible: self.visible,
            quota_units: self.quota_units,
            cipher: self.cipher.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy binding template in status created.
pub fn new_legacy_binding_template(code: &str, title: &str) -> LegacyBindingTemplate {
    LegacyBindingTemplate {
        code: code.to_string(),
        title: title.to_string(),
        visible: false,
        quota_units: 0,
        cipher: "".to_string(),
        issuer: "".to_string(),
        status: LegacyBindingTemplateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_legacy_binding_template("code-314", "title-490");
        assert_eq!(item.status, LegacyBindingTemplateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_legacy_binding_template("code-314", "title-490");
        if item.move_to(LegacyBindingTemplateStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyBindingTemplateStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_legacy_binding_template("code-314", "title-490");
        item.status = LegacyBindingTemplateStatus::Retired;
        assert!(
            item.move_to(LegacyBindingTemplateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
