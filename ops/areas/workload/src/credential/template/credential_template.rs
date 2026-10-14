use super::*;

use acme_kit::text;

/// Where a credential template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialTemplateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CredentialTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CredentialTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One credential template, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialTemplate {
    pub code: String,
    pub name: String,
    pub maximum: f64,
    pub cluster: String,
    pub ratio: f64,
    pub currency: String,
    pub version: i64,
    pub audience: String,
    pub locale: String,
    pub updated_at: i64,
    pub window_days: i64,
    pub status: CredentialTemplateStatus,
}

impl CredentialTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialTemplateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialTemplateStatus) -> bool {
        match self.status {
            CredentialTemplateStatus::Created => {
                return next == CredentialTemplateStatus::Published;
            }
            CredentialTemplateStatus::Validated => {
                return next == CredentialTemplateStatus::Retired;
            }
            CredentialTemplateStatus::Published => {
                return next == CredentialTemplateStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.window_days as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialTemplate {
        CredentialTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            maximum: self.maximum,
            cluster: self.cluster.clone(),
            ratio: self.ratio,
            currency: self.currency.clone(),
            version: self.version,
            audience: self.audience.clone(),
            locale: self.locale.clone(),
            updated_at: self.updated_at,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a credential template in status created.
pub fn new_credential_template(code: &str, name: &str) -> CredentialTemplate {
    CredentialTemplate {
        code: code.to_string(),
        name: name.to_string(),
        maximum: 0.0,
        cluster: "".to_string(),
        ratio: 0.0,
        currency: "".to_string(),
        version: 0,
        audience: "".to_string(),
        locale: "".to_string(),
        updated_at: 0,
        window_days: 0,
        status: CredentialTemplateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_credential_template("code-746", "name-999");
        assert_eq!(item.status, CredentialTemplateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_credential_template("code-746", "name-999");
        if item.move_to(CredentialTemplateStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialTemplateStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_credential_template("code-746", "name-999");
        item.status = CredentialTemplateStatus::Retired;
        assert!(
            item.move_to(CredentialTemplateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
