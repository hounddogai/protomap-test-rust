use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a rule notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleNoticeStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RuleNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RuleNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule notice as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleNotice {
    pub code: String,
    pub name: String,
    pub pinned: bool,
    pub limit: i64,
    pub fingerprint: String,
    pub port: i64,
    pub country: String,
    pub duration_minutes: i64,
    pub capacity: i64,
    pub attributes: BTreeMap<String, String>,
    pub checksum: String,
    pub status: RuleNoticeStatus,
}

impl RuleNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleNoticeStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleNoticeStatus) -> bool {
        match self.status {
            RuleNoticeStatus::Requested => {
                return next == RuleNoticeStatus::Granted || next == RuleNoticeStatus::Expired;
            }
            RuleNoticeStatus::Granted => {
                match next {
                    RuleNoticeStatus::Revoked | RuleNoticeStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleNoticeStatus::Revoked => {
                return next == RuleNoticeStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rule notice", self.code)
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

    pub fn projected_volume(&self) -> f64 {
        ((self.port as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleNotice {
        RuleNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            pinned: self.pinned,
            limit: self.limit,
            fingerprint: self.fingerprint.clone(),
            port: self.port,
            country: self.country.clone(),
            duration_minutes: self.duration_minutes,
            capacity: self.capacity,
            attributes: self.attributes.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a rule notice in status requested.
pub fn new_rule_notice(code: &str, name: &str) -> RuleNotice {
    RuleNotice {
        code: code.to_string(),
        name: name.to_string(),
        pinned: false,
        limit: 0,
        fingerprint: "".to_string(),
        port: 0,
        country: "".to_string(),
        duration_minutes: 0,
        capacity: 0,
        attributes: BTreeMap::new(),
        checksum: "".to_string(),
        status: RuleNoticeStatus::Requested,
    }
}
