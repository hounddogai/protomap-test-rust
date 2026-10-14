use super::*;

use acme_kit::text;

/// Where a rolling peering audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingPeeringAuditStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingPeeringAuditStatus {
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

impl std::fmt::Display for RollingPeeringAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling peering audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringAudit {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub channel: String,
    pub rotation_days: i64,
    pub batch_size: i64,
    pub tags: Vec<String>,
    pub status: RollingPeeringAuditStatus,
}

impl RollingPeeringAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingPeeringAuditStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingPeeringAuditStatus) -> bool {
        match self.status {
            RollingPeeringAuditStatus::Draft => {
                match next {
                    RollingPeeringAuditStatus::Active | RollingPeeringAuditStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingPeeringAuditStatus::Pending => {
                return next == RollingPeeringAuditStatus::Suspended || next == RollingPeeringAuditStatus::Closed;
            }
            RollingPeeringAuditStatus::Active => {
                return next == RollingPeeringAuditStatus::Closed;
            }
            RollingPeeringAuditStatus::Suspended => {
                return next == RollingPeeringAuditStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingPeeringAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling peering audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling peering audit", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_volume(&self) -> f64 {
        ((self.rotation_days as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingPeeringAudit {
        RollingPeeringAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            channel: self.channel.clone(),
            rotation_days: self.rotation_days,
            batch_size: self.batch_size,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a rolling peering audit in status draft.
pub fn new_rolling_peering_audit(code: &str, name: &str) -> RollingPeeringAudit {
    RollingPeeringAudit {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        channel: "".to_string(),
        rotation_days: 0,
        batch_size: 0,
        tags: Vec::new(),
        status: RollingPeeringAuditStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rolling_peering_audit("code-148", "name-599");
        assert_eq!(item.status, RollingPeeringAuditStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_rolling_peering_audit("code-148", "name-599");
        if item.move_to(RollingPeeringAuditStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingPeeringAuditStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rolling_peering_audit("code-148", "name-599");
        item.status = RollingPeeringAuditStatus::Closed;
        assert!(
            item.move_to(RollingPeeringAuditStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
