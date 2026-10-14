use super::*;

use acme_kit::text;

/// Where a tenant profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantProfileStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TenantProfileStatus {
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

impl std::fmt::Display for TenantProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tenant profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantProfile {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub pinned: bool,
    pub margin: f64,
    pub maximum: f64,
    pub weight: f64,
    pub status: TenantProfileStatus,
}

impl TenantProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantProfileStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantProfileStatus) -> bool {
        match self.status {
            TenantProfileStatus::New => {
                match next {
                    TenantProfileStatus::Queued | TenantProfileStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantProfileStatus::Queued => {
                match next {
                    TenantProfileStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantProfileStatus::Running => {
                match next {
                    TenantProfileStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantProfileStatus::Done => {
                return next == TenantProfileStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TenantProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tenant profile", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.maximum - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantProfile {
        TenantProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            pinned: self.pinned,
            margin: self.margin,
            maximum: self.maximum,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tenant profile in status new.
pub fn new_tenant_profile(code: &str, label: &str) -> TenantProfile {
    TenantProfile {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        pinned: false,
        margin: 0.0,
        maximum: 0.0,
        weight: 0.0,
        status: TenantProfileStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_tenant_profile("code-906", "label-144");
        assert_eq!(item.status, TenantProfileStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_tenant_profile("code-906", "label-144");
        if item.move_to(TenantProfileStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantProfileStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_tenant_profile("code-906", "label-144");
        item.status = TenantProfileStatus::Failed;
        assert!(item.move_to(TenantProfileStatus::New).is_err(), "expected move to fail");
    }
}
