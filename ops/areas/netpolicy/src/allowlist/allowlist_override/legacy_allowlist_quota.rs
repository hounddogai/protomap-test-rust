use super::*;

use acme_kit::text;

/// Where a legacy allowlist quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyAllowlistQuotaStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyAllowlistQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for LegacyAllowlistQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy allowlist quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistQuota {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub visible: bool,
    pub phone: String,
    pub minimum: f64,
    pub grace_days: i64,
    pub checksum: String,
    pub notes: String,
    pub budget: f64,
    pub status: LegacyAllowlistQuotaStatus,
}

impl LegacyAllowlistQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyAllowlistQuotaStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyAllowlistQuotaStatus) -> bool {
        match self.status {
            LegacyAllowlistQuotaStatus::New => {
                match next {
                    LegacyAllowlistQuotaStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyAllowlistQuotaStatus::Queued => {
                match next {
                    LegacyAllowlistQuotaStatus::Running | LegacyAllowlistQuotaStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyAllowlistQuotaStatus::Running => {
                match next {
                    LegacyAllowlistQuotaStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyAllowlistQuotaStatus::Done => {
                match next {
                    LegacyAllowlistQuotaStatus::Failed => {
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

    pub fn move_to(&mut self, next: LegacyAllowlistQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy allowlist quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy allowlist quota", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.budget + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyAllowlistQuota {
        LegacyAllowlistQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            visible: self.visible,
            phone: self.phone.clone(),
            minimum: self.minimum,
            grace_days: self.grace_days,
            checksum: self.checksum.clone(),
            notes: self.notes.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy allowlist quota in status new.
pub fn new_legacy_allowlist_quota(code: &str, title: &str) -> LegacyAllowlistQuota {
    LegacyAllowlistQuota {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        visible: false,
        phone: "".to_string(),
        minimum: 0.0,
        grace_days: 0,
        checksum: "".to_string(),
        notes: "".to_string(),
        budget: 0.0,
        status: LegacyAllowlistQuotaStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_legacy_allowlist_quota("code-71", "title-580");
        assert_eq!(item.status, LegacyAllowlistQuotaStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_legacy_allowlist_quota("code-71", "title-580");
        if item.move_to(LegacyAllowlistQuotaStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyAllowlistQuotaStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_legacy_allowlist_quota("code-71", "title-580");
        item.status = LegacyAllowlistQuotaStatus::Failed;
        assert!(
            item.move_to(LegacyAllowlistQuotaStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
