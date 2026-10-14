use super::*;

use acme_kit::text;

/// Where a primary rule notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRuleNoticeStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryRuleNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryRuleNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary rule notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleNotice {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub country: String,
    pub description: String,
    pub maximum: f64,
    pub visible: bool,
    pub active: bool,
    pub starts_at: i64,
    pub tags: Vec<String>,
    pub status: PrimaryRuleNoticeStatus,
}

impl PrimaryRuleNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRuleNoticeStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRuleNoticeStatus) -> bool {
        match self.status {
            PrimaryRuleNoticeStatus::Requested => {
                return next == PrimaryRuleNoticeStatus::Granted || next == PrimaryRuleNoticeStatus::Expired;
            }
            PrimaryRuleNoticeStatus::Granted => {
                match next {
                    PrimaryRuleNoticeStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryRuleNoticeStatus::Revoked => {
                match next {
                    PrimaryRuleNoticeStatus::Expired => {
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

    pub fn move_to(&mut self, next: PrimaryRuleNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary rule notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_value(&self) -> f64 {
        (self.maximum * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryRuleNotice {
        PrimaryRuleNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            country: self.country.clone(),
            description: self.description.clone(),
            maximum: self.maximum,
            visible: self.visible,
            active: self.active,
            starts_at: self.starts_at,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a primary rule notice in status requested.
pub fn new_primary_rule_notice(code: &str, title: &str) -> PrimaryRuleNotice {
    PrimaryRuleNotice {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        country: "".to_string(),
        description: "".to_string(),
        maximum: 0.0,
        visible: false,
        active: false,
        starts_at: 0,
        tags: Vec::new(),
        status: PrimaryRuleNoticeStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_primary_rule_notice("code-236", "title-161");
        assert_eq!(item.status, PrimaryRuleNoticeStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_primary_rule_notice("code-236", "title-161");
        if item.move_to(PrimaryRuleNoticeStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryRuleNoticeStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_primary_rule_notice("code-236", "title-161");
        item.status = PrimaryRuleNoticeStatus::Expired;
        assert!(
            item.move_to(PrimaryRuleNoticeStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
