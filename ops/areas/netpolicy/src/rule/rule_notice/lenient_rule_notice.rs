use super::*;

use acme_kit::text;

/// Where a lenient rule notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientRuleNoticeStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientRuleNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientRuleNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient rule notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleNotice {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub key_id: String,
    pub capacity: i64,
    pub budget: f64,
    pub limit: i64,
    pub status: LenientRuleNoticeStatus,
}

impl LenientRuleNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientRuleNoticeStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientRuleNoticeStatus) -> bool {
        match self.status {
            LenientRuleNoticeStatus::Requested => {
                match next {
                    LenientRuleNoticeStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRuleNoticeStatus::Granted => {
                return next == LenientRuleNoticeStatus::Expired;
            }
            LenientRuleNoticeStatus::Revoked => {
                match next {
                    LenientRuleNoticeStatus::Expired => {
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

    pub fn move_to(&mut self, next: LenientRuleNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient rule notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.weight + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientRuleNotice {
        LenientRuleNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            key_id: self.key_id.clone(),
            capacity: self.capacity,
            budget: self.budget,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient rule notice in status requested.
pub fn new_lenient_rule_notice(code: &str, title: &str) -> LenientRuleNotice {
    LenientRuleNotice {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        key_id: "".to_string(),
        capacity: 0,
        budget: 0.0,
        limit: 0,
        status: LenientRuleNoticeStatus::Requested,
    }
}
