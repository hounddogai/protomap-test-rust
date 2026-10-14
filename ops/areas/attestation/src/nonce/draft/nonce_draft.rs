use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a nonce draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceDraftStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl NonceDraftStatus {
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

impl std::fmt::Display for NonceDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceDraft {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub namespace: String,
    pub contact_email: String,
    pub attributes: BTreeMap<String, String>,
    pub quantity: i64,
    pub status: NonceDraftStatus,
}

impl NonceDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceDraftStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceDraftStatus) -> bool {
        match self.status {
            NonceDraftStatus::Idle => {
                match next {
                    NonceDraftStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceDraftStatus::Warming => {
                return next == NonceDraftStatus::Draining || next == NonceDraftStatus::Stopped;
            }
            NonceDraftStatus::Ready => {
                return next == NonceDraftStatus::Draining || next == NonceDraftStatus::Stopped;
            }
            NonceDraftStatus::Draining => {
                return next == NonceDraftStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: NonceDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
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
        self.quantity as f64
    }

    pub fn copy(&self) -> NonceDraft {
        NonceDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            namespace: self.namespace.clone(),
            contact_email: self.contact_email.clone(),
            attributes: self.attributes.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a nonce draft in status idle.
pub fn new_nonce_draft(code: &str, label: &str) -> NonceDraft {
    NonceDraft {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        namespace: "".to_string(),
        contact_email: "".to_string(),
        attributes: BTreeMap::new(),
        quantity: 0,
        status: NonceDraftStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_nonce_draft("code-182", "label-823");
        assert_eq!(item.status, NonceDraftStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_nonce_draft("code-182", "label-823");
        if item.move_to(NonceDraftStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceDraftStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_nonce_draft("code-182", "label-823");
        item.status = NonceDraftStatus::Stopped;
        assert!(item.move_to(NonceDraftStatus::Idle).is_err(), "expected move to fail");
    }
}
