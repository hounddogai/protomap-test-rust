use super::*;

use acme_kit::text;

/// Where a manual credential score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualCredentialScoreStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ManualCredentialScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ManualCredentialScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual credential score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialScore {
    pub code: String,
    pub label: String,
    pub title: String,
    pub category: String,
    pub discount: f64,
    pub floor_value: f64,
    pub name: String,
    pub status: ManualCredentialScoreStatus,
}

impl ManualCredentialScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualCredentialScoreStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualCredentialScoreStatus) -> bool {
        match self.status {
            ManualCredentialScoreStatus::Created => {
                match next {
                    ManualCredentialScoreStatus::Validated | ManualCredentialScoreStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualCredentialScoreStatus::Validated => {
                return next == ManualCredentialScoreStatus::Published || next == ManualCredentialScoreStatus::Retired;
            }
            ManualCredentialScoreStatus::Published => {
                match next {
                    ManualCredentialScoreStatus::Retired => {
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

    pub fn move_to(&mut self, next: ManualCredentialScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual credential score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.floor_value + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualCredentialScore {
        ManualCredentialScore {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            discount: self.discount,
            floor_value: self.floor_value,
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a manual credential score in status created.
pub fn new_manual_credential_score(code: &str, label: &str) -> ManualCredentialScore {
    ManualCredentialScore {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        category: "".to_string(),
        discount: 0.0,
        floor_value: 0.0,
        name: "".to_string(),
        status: ManualCredentialScoreStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_manual_credential_score("code-306", "label-249");
        assert_eq!(item.status, ManualCredentialScoreStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_manual_credential_score("code-306", "label-249");
        if item.move_to(ManualCredentialScoreStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualCredentialScoreStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_manual_credential_score("code-306", "label-249");
        item.status = ManualCredentialScoreStatus::Retired;
        assert!(
            item.move_to(ManualCredentialScoreStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
