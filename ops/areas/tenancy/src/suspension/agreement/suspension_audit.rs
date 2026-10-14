use super::*;

use acme_kit::text;

/// Where a suspension audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionAuditStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SuspensionAuditStatus {
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

impl std::fmt::Display for SuspensionAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a suspension audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAudit {
    pub code: String,
    pub name: String,
    pub key_id: String,
    pub port: i64,
    pub region: String,
    pub budget: f64,
    pub locale: String,
    pub status: SuspensionAuditStatus,
}

impl SuspensionAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionAuditStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionAuditStatus) -> bool {
        match self.status {
            SuspensionAuditStatus::Idle => {
                return next == SuspensionAuditStatus::Stopped;
            }
            SuspensionAuditStatus::Warming => {
                match next {
                    SuspensionAuditStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionAuditStatus::Ready => {
                match next {
                    SuspensionAuditStatus::Draining | SuspensionAuditStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionAuditStatus::Draining => {
                match next {
                    SuspensionAuditStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SuspensionAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "suspension audit", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.budget + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionAudit {
        SuspensionAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            key_id: self.key_id.clone(),
            port: self.port,
            region: self.region.clone(),
            budget: self.budget,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a suspension audit in status idle.
pub fn new_suspension_audit(code: &str, name: &str) -> SuspensionAudit {
    SuspensionAudit {
        code: code.to_string(),
        name: name.to_string(),
        key_id: "".to_string(),
        port: 0,
        region: "".to_string(),
        budget: 0.0,
        locale: "".to_string(),
        status: SuspensionAuditStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_suspension_audit("code-396", "name-680");
        assert_eq!(item.status, SuspensionAuditStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_suspension_audit("code-396", "name-680");
        if item.move_to(SuspensionAuditStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionAuditStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_suspension_audit("code-396", "name-680");
        item.status = SuspensionAuditStatus::Stopped;
        assert!(
            item.move_to(SuspensionAuditStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
