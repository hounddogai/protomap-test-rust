use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a freeze template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeTemplateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FreezeTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FreezeTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeTemplate {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub attributes: BTreeMap<String, String>,
    pub currency: String,
    pub status: FreezeTemplateStatus,
}

impl FreezeTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeTemplateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeTemplateStatus) -> bool {
        match self.status {
            FreezeTemplateStatus::Requested => {
                return next == FreezeTemplateStatus::Granted;
            }
            FreezeTemplateStatus::Granted => {
                return next == FreezeTemplateStatus::Revoked || next == FreezeTemplateStatus::Expired;
            }
            FreezeTemplateStatus::Revoked => {
                match next {
                    FreezeTemplateStatus::Expired => {
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

    pub fn move_to(&mut self, next: FreezeTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
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

    pub fn measured_value(&self) -> f64 {
        self.burst as f64
    }

    pub fn copy(&self) -> FreezeTemplate {
        FreezeTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            attributes: self.attributes.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a freeze template in status requested.
pub fn new_freeze_template(code: &str, label: &str) -> FreezeTemplate {
    FreezeTemplate {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        attributes: BTreeMap::new(),
        currency: "".to_string(),
        status: FreezeTemplateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_freeze_template("code-486", "label-159");
        assert_eq!(item.status, FreezeTemplateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_freeze_template("code-486", "label-159");
        if item.move_to(FreezeTemplateStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeTemplateStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_freeze_template("code-486", "label-159");
        item.status = FreezeTemplateStatus::Expired;
        assert!(
            item.move_to(FreezeTemplateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
