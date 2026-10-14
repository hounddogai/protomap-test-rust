use super::*;

use acme_kit::text;

/// Where a plan digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanDigestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PlanDigestStatus {
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

impl std::fmt::Display for PlanDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanDigest {
    pub code: String,
    pub name: String,
    pub pinned: bool,
    pub port: i64,
    pub unit: String,
    pub region: String,
    pub description: String,
    pub burst: i64,
    pub ttl_seconds: i64,
    pub starts_at: i64,
    pub visible: bool,
    pub status: PlanDigestStatus,
}

impl PlanDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanDigestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanDigestStatus) -> bool {
        match self.status {
            PlanDigestStatus::Draft => {
                match next {
                    PlanDigestStatus::Active | PlanDigestStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanDigestStatus::Pending => {
                return next == PlanDigestStatus::Active;
            }
            PlanDigestStatus::Active => {
                return next == PlanDigestStatus::Suspended || next == PlanDigestStatus::Closed;
            }
            PlanDigestStatus::Suspended => {
                return next == PlanDigestStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PlanDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanDigest {
        PlanDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            pinned: self.pinned,
            port: self.port,
            unit: self.unit.clone(),
            region: self.region.clone(),
            description: self.description.clone(),
            burst: self.burst,
            ttl_seconds: self.ttl_seconds,
            starts_at: self.starts_at,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a plan digest in status draft.
pub fn new_plan_digest(code: &str, name: &str) -> PlanDigest {
    PlanDigest {
        code: code.to_string(),
        name: name.to_string(),
        pinned: false,
        port: 0,
        unit: "".to_string(),
        region: "".to_string(),
        description: "".to_string(),
        burst: 0,
        ttl_seconds: 0,
        starts_at: 0,
        visible: false,
        status: PlanDigestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_plan_digest("code-674", "name-584");
        assert_eq!(item.status, PlanDigestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_plan_digest("code-674", "name-584");
        if item.move_to(PlanDigestStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanDigestStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_plan_digest("code-674", "name-584");
        item.status = PlanDigestStatus::Closed;
        assert!(item.move_to(PlanDigestStatus::Draft).is_err(), "expected move to fail");
    }
}
