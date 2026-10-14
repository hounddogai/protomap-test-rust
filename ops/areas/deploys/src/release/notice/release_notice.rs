use super::*;

use acme_kit::text;

/// Where a release notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseNoticeStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ReleaseNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ReleaseNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One release notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseNotice {
    pub code: String,
    pub label: String,
    pub category: String,
    pub retries: i64,
    pub amount: f64,
    pub discount: f64,
    pub score: f64,
    pub region: String,
    pub status: ReleaseNoticeStatus,
}

impl ReleaseNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseNoticeStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseNoticeStatus) -> bool {
        match self.status {
            ReleaseNoticeStatus::Draft => {
                return next == ReleaseNoticeStatus::Pending;
            }
            ReleaseNoticeStatus::Pending => {
                return next == ReleaseNoticeStatus::Suspended || next == ReleaseNoticeStatus::Closed;
            }
            ReleaseNoticeStatus::Active => {
                return next == ReleaseNoticeStatus::Suspended;
            }
            ReleaseNoticeStatus::Suspended => {
                match next {
                    ReleaseNoticeStatus::Closed => {
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

    pub fn move_to(&mut self, next: ReleaseNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.retries as f64) * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseNotice {
        ReleaseNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            retries: self.retries,
            amount: self.amount,
            discount: self.discount,
            score: self.score,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a release notice in status draft.
pub fn new_release_notice(code: &str, label: &str) -> ReleaseNotice {
    ReleaseNotice {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        retries: 0,
        amount: 0.0,
        discount: 0.0,
        score: 0.0,
        region: "".to_string(),
        status: ReleaseNoticeStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_release_notice("code-657", "label-691");
        assert_eq!(item.status, ReleaseNoticeStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_release_notice("code-657", "label-691");
        if item.move_to(ReleaseNoticeStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseNoticeStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_release_notice("code-657", "label-691");
        item.status = ReleaseNoticeStatus::Closed;
        assert!(
            item.move_to(ReleaseNoticeStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
