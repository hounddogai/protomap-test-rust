use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a federation agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationAgreementStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FederationAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for FederationAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One federation agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationAgreement {
    pub code: String,
    pub name: String,
    pub category: String,
    pub minimum: f64,
    pub cipher: String,
    pub discount: f64,
    pub attributes: BTreeMap<String, String>,
    pub status: FederationAgreementStatus,
}

impl FederationAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationAgreementStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationAgreementStatus) -> bool {
        match self.status {
            FederationAgreementStatus::New => {
                return next == FederationAgreementStatus::Failed;
            }
            FederationAgreementStatus::Queued => {
                return next == FederationAgreementStatus::Failed;
            }
            FederationAgreementStatus::Running => {
                match next {
                    FederationAgreementStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationAgreementStatus::Done => {
                match next {
                    FederationAgreementStatus::Failed => {
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

    pub fn move_to(&mut self, next: FederationAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
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

    pub fn projected_value(&self) -> f64 {
        (self.minimum * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationAgreement {
        FederationAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            minimum: self.minimum,
            cipher: self.cipher.clone(),
            discount: self.discount,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a federation agreement in status new.
pub fn new_federation_agreement(code: &str, name: &str) -> FederationAgreement {
    FederationAgreement {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        minimum: 0.0,
        cipher: "".to_string(),
        discount: 0.0,
        attributes: BTreeMap::new(),
        status: FederationAgreementStatus::New,
    }
}
