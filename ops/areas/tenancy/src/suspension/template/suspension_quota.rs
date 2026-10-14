use super::*;

use acme_kit::text;

/// Where a suspension quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionQuotaStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SuspensionQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for SuspensionQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension quota as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionQuota {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub source: String,
    pub archived: bool,
    pub port: i64,
    pub quota_units: i64,
    pub category: String,
    pub name: String,
    pub unit: String,
    pub status: SuspensionQuotaStatus,
}

impl SuspensionQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionQuotaStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionQuotaStatus) -> bool {
        match self.status {
            SuspensionQuotaStatus::Idle => {
                match next {
                    SuspensionQuotaStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionQuotaStatus::Warming => {
                return next == SuspensionQuotaStatus::Draining || next == SuspensionQuotaStatus::Stopped;
            }
            SuspensionQuotaStatus::Ready => {
                match next {
                    SuspensionQuotaStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionQuotaStatus::Draining => {
                match next {
                    SuspensionQuotaStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SuspensionQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "suspension quota", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.port as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionQuota {
        SuspensionQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            source: self.source.clone(),
            archived: self.archived,
            port: self.port,
            quota_units: self.quota_units,
            category: self.category.clone(),
            name: self.name.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a suspension quota in status idle.
pub fn new_suspension_quota(code: &str, label: &str) -> SuspensionQuota {
    SuspensionQuota {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        source: "".to_string(),
        archived: false,
        port: 0,
        quota_units: 0,
        category: "".to_string(),
        name: "".to_string(),
        unit: "".to_string(),
        status: SuspensionQuotaStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_suspension_quota("code-18", "label-448");
        assert_eq!(item.status, SuspensionQuotaStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_suspension_quota("code-18", "label-448");
        if item.move_to(SuspensionQuotaStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionQuotaStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_suspension_quota("code-18", "label-448");
        item.status = SuspensionQuotaStatus::Stopped;
        assert!(
            item.move_to(SuspensionQuotaStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
