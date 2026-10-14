use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a strict approval notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictApprovalNoticeStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl StrictApprovalNoticeStatus {
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

impl std::fmt::Display for StrictApprovalNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict approval notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalNotice {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub cipher: String,
    pub updated_at: i64,
    pub timezone: String,
    pub notes: String,
    pub cluster: String,
    pub minimum: f64,
    pub cost: f64,
    pub status: StrictApprovalNoticeStatus,
}

impl StrictApprovalNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictApprovalNoticeStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictApprovalNoticeStatus) -> bool {
        match self.status {
            StrictApprovalNoticeStatus::New => {
                return next == StrictApprovalNoticeStatus::Queued;
            }
            StrictApprovalNoticeStatus::Queued => {
                return next == StrictApprovalNoticeStatus::Running || next == StrictApprovalNoticeStatus::Failed;
            }
            StrictApprovalNoticeStatus::Running => {
                match next {
                    StrictApprovalNoticeStatus::Done | StrictApprovalNoticeStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictApprovalNoticeStatus::Done => {
                return next == StrictApprovalNoticeStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictApprovalNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict approval notice {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
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

    pub fn projected_total(&self) -> f64 {
        (self.minimum * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictApprovalNotice {
        StrictApprovalNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            cipher: self.cipher.clone(),
            updated_at: self.updated_at,
            timezone: self.timezone.clone(),
            notes: self.notes.clone(),
            cluster: self.cluster.clone(),
            minimum: self.minimum,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a strict approval notice in status new.
pub fn new_strict_approval_notice(code: &str, title: &str) -> StrictApprovalNotice {
    StrictApprovalNotice {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        cipher: "".to_string(),
        updated_at: 0,
        timezone: "".to_string(),
        notes: "".to_string(),
        cluster: "".to_string(),
        minimum: 0.0,
        cost: 0.0,
        status: StrictApprovalNoticeStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_strict_approval_notice("code-694", "title-384");
        assert_eq!(item.status, StrictApprovalNoticeStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_strict_approval_notice("code-694", "title-384");
        if item.move_to(StrictApprovalNoticeStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictApprovalNoticeStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_strict_approval_notice("code-694", "title-384");
        item.status = StrictApprovalNoticeStatus::Failed;
        assert!(
            item.move_to(StrictApprovalNoticeStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
