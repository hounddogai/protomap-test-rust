use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a zone agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneAgreementStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ZoneAgreementStatus {
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

impl std::fmt::Display for ZoneAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One zone agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneAgreement {
    pub code: String,
    pub title: String,
    pub locale: String,
    pub updated_at: i64,
    pub external_ref: String,
    pub retries: i64,
    pub attributes: BTreeMap<String, String>,
    pub sequence: i64,
    pub pinned: bool,
    pub status: ZoneAgreementStatus,
}

impl ZoneAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneAgreementStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneAgreementStatus) -> bool {
        match self.status {
            ZoneAgreementStatus::Idle => {
                match next {
                    ZoneAgreementStatus::Ready | ZoneAgreementStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneAgreementStatus::Warming => {
                return next == ZoneAgreementStatus::Ready;
            }
            ZoneAgreementStatus::Ready => {
                match next {
                    ZoneAgreementStatus::Draining | ZoneAgreementStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneAgreementStatus::Draining => {
                return next == ZoneAgreementStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ZoneAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "zone agreement", self.code)
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

    pub fn adjusted_load(&self) -> f64 {
        ((self.sequence as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneAgreement {
        ZoneAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            updated_at: self.updated_at,
            external_ref: self.external_ref.clone(),
            retries: self.retries,
            attributes: self.attributes.clone(),
            sequence: self.sequence,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a zone agreement in status idle.
pub fn new_zone_agreement(code: &str, title: &str) -> ZoneAgreement {
    ZoneAgreement {
        code: code.to_string(),
        title: title.to_string(),
        locale: "".to_string(),
        updated_at: 0,
        external_ref: "".to_string(),
        retries: 0,
        attributes: BTreeMap::new(),
        sequence: 0,
        pinned: false,
        status: ZoneAgreementStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_zone_agreement("code-989", "title-26");
        assert_eq!(item.status, ZoneAgreementStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_zone_agreement("code-989", "title-26");
        if item.move_to(ZoneAgreementStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneAgreementStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_zone_agreement("code-989", "title-26");
        item.status = ZoneAgreementStatus::Stopped;
        assert!(
            item.move_to(ZoneAgreementStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
