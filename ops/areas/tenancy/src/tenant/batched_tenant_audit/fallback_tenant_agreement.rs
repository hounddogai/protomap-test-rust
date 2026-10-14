use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a fallback tenant agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackTenantAgreementStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FallbackTenantAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for FallbackTenantAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback tenant agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTenantAgreement {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub attributes: BTreeMap<String, String>,
    pub cipher: String,
    pub timezone: String,
    pub currency: String,
    pub visible: bool,
    pub fingerprint: String,
    pub audience: String,
    pub status: FallbackTenantAgreementStatus,
}

impl FallbackTenantAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackTenantAgreementStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackTenantAgreementStatus) -> bool {
        match self.status {
            FallbackTenantAgreementStatus::Idle => {
                match next {
                    FallbackTenantAgreementStatus::Warming | FallbackTenantAgreementStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackTenantAgreementStatus::Warming => {
                match next {
                    FallbackTenantAgreementStatus::Draining | FallbackTenantAgreementStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackTenantAgreementStatus::Ready => {
                return next == FallbackTenantAgreementStatus::Draining
                    || next == FallbackTenantAgreementStatus::Stopped;
            }
            FallbackTenantAgreementStatus::Draining => {
                return next == FallbackTenantAgreementStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackTenantAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback tenant agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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
        self.margin
    }

    pub fn copy(&self) -> FallbackTenantAgreement {
        FallbackTenantAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            attributes: self.attributes.clone(),
            cipher: self.cipher.clone(),
            timezone: self.timezone.clone(),
            currency: self.currency.clone(),
            visible: self.visible,
            fingerprint: self.fingerprint.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a fallback tenant agreement in status idle.
pub fn new_fallback_tenant_agreement(code: &str, label: &str) -> FallbackTenantAgreement {
    FallbackTenantAgreement {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        attributes: BTreeMap::new(),
        cipher: "".to_string(),
        timezone: "".to_string(),
        currency: "".to_string(),
        visible: false,
        fingerprint: "".to_string(),
        audience: "".to_string(),
        status: FallbackTenantAgreementStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_fallback_tenant_agreement("code-424", "label-773");
        assert_eq!(item.status, FallbackTenantAgreementStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_fallback_tenant_agreement("code-424", "label-773");
        if item.move_to(FallbackTenantAgreementStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackTenantAgreementStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_fallback_tenant_agreement("code-424", "label-773");
        item.status = FallbackTenantAgreementStatus::Stopped;
        assert!(
            item.move_to(FallbackTenantAgreementStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
