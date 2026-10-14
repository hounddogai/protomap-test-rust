use super::*;

use acme_kit::text;

/// Where a verifier entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierEntryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl VerifierEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for VerifierEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a verifier entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierEntry {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub source: String,
    pub summary: String,
    pub active: bool,
    pub external_ref: String,
    pub region: String,
    pub weight: f64,
    pub locale: String,
    pub segment: String,
    pub status: VerifierEntryStatus,
}

impl VerifierEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierEntryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierEntryStatus) -> bool {
        match self.status {
            VerifierEntryStatus::Requested => {
                match next {
                    VerifierEntryStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierEntryStatus::Granted => {
                return next == VerifierEntryStatus::Revoked || next == VerifierEntryStatus::Expired;
            }
            VerifierEntryStatus::Revoked => {
                return next == VerifierEntryStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "verifier entry", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.weight
    }

    pub fn copy(&self) -> VerifierEntry {
        VerifierEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            source: self.source.clone(),
            summary: self.summary.clone(),
            active: self.active,
            external_ref: self.external_ref.clone(),
            region: self.region.clone(),
            weight: self.weight,
            locale: self.locale.clone(),
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a verifier entry in status requested.
pub fn new_verifier_entry(code: &str, label: &str) -> VerifierEntry {
    VerifierEntry {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        source: "".to_string(),
        summary: "".to_string(),
        active: false,
        external_ref: "".to_string(),
        region: "".to_string(),
        weight: 0.0,
        locale: "".to_string(),
        segment: "".to_string(),
        status: VerifierEntryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_verifier_entry("code-880", "label-871");
        assert_eq!(item.status, VerifierEntryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_verifier_entry("code-880", "label-871");
        if item.move_to(VerifierEntryStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierEntryStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_verifier_entry("code-880", "label-871");
        item.status = VerifierEntryStatus::Expired;
        assert!(
            item.move_to(VerifierEntryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
