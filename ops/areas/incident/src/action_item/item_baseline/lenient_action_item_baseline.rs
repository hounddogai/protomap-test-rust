use super::*;

use acme_kit::text;

/// Where a lenient action item baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientActionItemBaselineStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientActionItemBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientActionItemBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient action item baseline and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemBaseline {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub burst: i64,
    pub host: String,
    pub country: String,
    pub grace_days: i64,
    pub unit: String,
    pub checksum: String,
    pub status: LenientActionItemBaselineStatus,
}

impl LenientActionItemBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientActionItemBaselineStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientActionItemBaselineStatus) -> bool {
        match self.status {
            LenientActionItemBaselineStatus::Requested => {
                match next {
                    LenientActionItemBaselineStatus::Revoked | LenientActionItemBaselineStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientActionItemBaselineStatus::Granted => {
                return next == LenientActionItemBaselineStatus::Revoked
                    || next == LenientActionItemBaselineStatus::Expired;
            }
            LenientActionItemBaselineStatus::Revoked => {
                return next == LenientActionItemBaselineStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientActionItemBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient action item baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.grace_days as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientActionItemBaseline {
        LenientActionItemBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            burst: self.burst,
            host: self.host.clone(),
            country: self.country.clone(),
            grace_days: self.grace_days,
            unit: self.unit.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a lenient action item baseline in status requested.
pub fn new_lenient_action_item_baseline(code: &str, label: &str) -> LenientActionItemBaseline {
    LenientActionItemBaseline {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        burst: 0,
        host: "".to_string(),
        country: "".to_string(),
        grace_days: 0,
        unit: "".to_string(),
        checksum: "".to_string(),
        status: LenientActionItemBaselineStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_lenient_action_item_baseline("code-43", "label-905");
        assert_eq!(item.status, LenientActionItemBaselineStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_lenient_action_item_baseline("code-43", "label-905");
        if item.move_to(LenientActionItemBaselineStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientActionItemBaselineStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_lenient_action_item_baseline("code-43", "label-905");
        item.status = LenientActionItemBaselineStatus::Expired;
        assert!(
            item.move_to(LenientActionItemBaselineStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
