use super::*;

use acme_kit::text;

/// Where a global sink quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalSinkQuotaStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GlobalSinkQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GlobalSinkQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global sink quota as the sink team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkQuota {
    pub code: String,
    pub label: String,
    pub title: String,
    pub ratio: f64,
    pub discount: f64,
    pub status: GlobalSinkQuotaStatus,
}

impl GlobalSinkQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalSinkQuotaStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalSinkQuotaStatus) -> bool {
        match self.status {
            GlobalSinkQuotaStatus::Requested => {
                return next == GlobalSinkQuotaStatus::Granted || next == GlobalSinkQuotaStatus::Revoked;
            }
            GlobalSinkQuotaStatus::Granted => {
                return next == GlobalSinkQuotaStatus::Revoked;
            }
            GlobalSinkQuotaStatus::Revoked => {
                return next == GlobalSinkQuotaStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalSinkQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global sink quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.discount + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalSinkQuota {
        GlobalSinkQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global sink quota in status requested.
pub fn new_global_sink_quota(code: &str, label: &str) -> GlobalSinkQuota {
    GlobalSinkQuota {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        ratio: 0.0,
        discount: 0.0,
        status: GlobalSinkQuotaStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_global_sink_quota("code-414", "label-723");
        assert_eq!(item.status, GlobalSinkQuotaStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_global_sink_quota("code-414", "label-723");
        if item.move_to(GlobalSinkQuotaStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalSinkQuotaStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_global_sink_quota("code-414", "label-723");
        item.status = GlobalSinkQuotaStatus::Expired;
        assert!(
            item.move_to(GlobalSinkQuotaStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
