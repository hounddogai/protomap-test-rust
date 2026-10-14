use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a approval quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalQuotaStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ApprovalQuotaStatus {
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

impl std::fmt::Display for ApprovalQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalQuota {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub fingerprint: String,
    pub ceiling: f64,
    pub channel: String,
    pub issuer: String,
    pub retries: i64,
    pub quota_units: i64,
    pub attributes: BTreeMap<String, String>,
    pub status: ApprovalQuotaStatus,
}

impl ApprovalQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalQuotaStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalQuotaStatus) -> bool {
        match self.status {
            ApprovalQuotaStatus::New => {
                match next {
                    ApprovalQuotaStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalQuotaStatus::Queued => {
                return next == ApprovalQuotaStatus::Running || next == ApprovalQuotaStatus::Done;
            }
            ApprovalQuotaStatus::Running => {
                return next == ApprovalQuotaStatus::Done || next == ApprovalQuotaStatus::Failed;
            }
            ApprovalQuotaStatus::Done => {
                match next {
                    ApprovalQuotaStatus::Failed => {
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

    pub fn move_to(&mut self, next: ApprovalQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
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

    pub fn net_total(&self) -> f64 {
        ((self.ceiling - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalQuota {
        ApprovalQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            fingerprint: self.fingerprint.clone(),
            ceiling: self.ceiling,
            channel: self.channel.clone(),
            issuer: self.issuer.clone(),
            retries: self.retries,
            quota_units: self.quota_units,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a approval quota in status new.
pub fn new_approval_quota(code: &str, name: &str) -> ApprovalQuota {
    ApprovalQuota {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        fingerprint: "".to_string(),
        ceiling: 0.0,
        channel: "".to_string(),
        issuer: "".to_string(),
        retries: 0,
        quota_units: 0,
        attributes: BTreeMap::new(),
        status: ApprovalQuotaStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_approval_quota("code-586", "name-267");
        assert_eq!(item.status, ApprovalQuotaStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_approval_quota("code-586", "name-267");
        if item.move_to(ApprovalQuotaStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalQuotaStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_approval_quota("code-586", "name-267");
        item.status = ApprovalQuotaStatus::Failed;
        assert!(item.move_to(ApprovalQuotaStatus::New).is_err(), "expected move to fail");
    }
}
