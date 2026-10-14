use super::*;

use acme_kit::text;

/// Where a regional retention notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalRetentionNoticeStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalRetentionNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalRetentionNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional retention notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionNotice {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub principal: String,
    pub locale: String,
    pub status: RegionalRetentionNoticeStatus,
}

impl RegionalRetentionNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalRetentionNoticeStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalRetentionNoticeStatus) -> bool {
        match self.status {
            RegionalRetentionNoticeStatus::Proposed => {
                match next {
                    RegionalRetentionNoticeStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRetentionNoticeStatus::Approved => {
                match next {
                    RegionalRetentionNoticeStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRetentionNoticeStatus::Rejected => {
                return next == RegionalRetentionNoticeStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalRetentionNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional retention notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.revision as f64
    }

    pub fn copy(&self) -> RegionalRetentionNotice {
        RegionalRetentionNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            principal: self.principal.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a regional retention notice in status proposed.
pub fn new_regional_retention_notice(code: &str, title: &str) -> RegionalRetentionNotice {
    RegionalRetentionNotice {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        principal: "".to_string(),
        locale: "".to_string(),
        status: RegionalRetentionNoticeStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_regional_retention_notice("code-236", "title-203");
        assert_eq!(item.status, RegionalRetentionNoticeStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_regional_retention_notice("code-236", "title-203");
        if item.move_to(RegionalRetentionNoticeStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalRetentionNoticeStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_regional_retention_notice("code-236", "title-203");
        item.status = RegionalRetentionNoticeStatus::Withdrawn;
        assert!(
            item.move_to(RegionalRetentionNoticeStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
