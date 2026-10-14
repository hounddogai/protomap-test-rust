use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a usage quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageQuotaStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl UsageQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for UsageQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A usage quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageQuota {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub attributes: BTreeMap<String, String>,
    pub host: String,
    pub amount: f64,
    pub discount: f64,
    pub status: UsageQuotaStatus,
}

impl UsageQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageQuotaStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageQuotaStatus) -> bool {
        match self.status {
            UsageQuotaStatus::Planned => {
                match next {
                    UsageQuotaStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageQuotaStatus::Scheduled => {
                return next == UsageQuotaStatus::InProgress || next == UsageQuotaStatus::Completed;
            }
            UsageQuotaStatus::InProgress => {
                match next {
                    UsageQuotaStatus::Completed | UsageQuotaStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageQuotaStatus::Completed => {
                return next == UsageQuotaStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "usage quota", self.code)
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

    pub fn net_exposure(&self) -> f64 {
        ((self.batch_size as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageQuota {
        UsageQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            attributes: self.attributes.clone(),
            host: self.host.clone(),
            amount: self.amount,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a usage quota in status planned.
pub fn new_usage_quota(code: &str, title: &str) -> UsageQuota {
    UsageQuota {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        attributes: BTreeMap::new(),
        host: "".to_string(),
        amount: 0.0,
        discount: 0.0,
        status: UsageQuotaStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_usage_quota("code-788", "title-690");
        assert_eq!(item.status, UsageQuotaStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_usage_quota("code-788", "title-690");
        if item.move_to(UsageQuotaStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageQuotaStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_usage_quota("code-788", "title-690");
        item.status = UsageQuotaStatus::Cancelled;
        assert!(
            item.move_to(UsageQuotaStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
