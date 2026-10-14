use super::*;

use acme_kit::text;

/// Where a alert agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertAgreementStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AlertAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AlertAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A alert agreement as the alert team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertAgreement {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub cipher: String,
    pub locale: String,
    pub external_ref: String,
    pub approved: bool,
    pub status: AlertAgreementStatus,
}

impl AlertAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertAgreementStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertAgreementStatus) -> bool {
        match self.status {
            AlertAgreementStatus::Created => {
                return next == AlertAgreementStatus::Published;
            }
            AlertAgreementStatus::Validated => {
                return next == AlertAgreementStatus::Published;
            }
            AlertAgreementStatus::Published => {
                return next == AlertAgreementStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AlertAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> AlertAgreement {
        AlertAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            cipher: self.cipher.clone(),
            locale: self.locale.clone(),
            external_ref: self.external_ref.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a alert agreement in status created.
pub fn new_alert_agreement(code: &str, name: &str) -> AlertAgreement {
    AlertAgreement {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        cipher: "".to_string(),
        locale: "".to_string(),
        external_ref: "".to_string(),
        approved: false,
        status: AlertAgreementStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_alert_agreement("code-961", "name-670");
        assert_eq!(item.status, AlertAgreementStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_alert_agreement("code-961", "name-670");
        if item.move_to(AlertAgreementStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertAgreementStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_alert_agreement("code-961", "name-670");
        item.status = AlertAgreementStatus::Retired;
        assert!(
            item.move_to(AlertAgreementStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
