use super::*;

use acme_kit::text;

/// Where a egress notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressNoticeStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EgressNoticeStatus {
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

impl std::fmt::Display for EgressNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A egress notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressNotice {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub owner_id: String,
    pub starts_at: i64,
    pub status: EgressNoticeStatus,
}

impl EgressNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressNoticeStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressNoticeStatus) -> bool {
        match self.status {
            EgressNoticeStatus::Draft => {
                return next == EgressNoticeStatus::Pending || next == EgressNoticeStatus::Active;
            }
            EgressNoticeStatus::Pending => {
                match next {
                    EgressNoticeStatus::Active | EgressNoticeStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressNoticeStatus::Active => {
                return next == EgressNoticeStatus::Suspended;
            }
            EgressNoticeStatus::Suspended => {
                match next {
                    EgressNoticeStatus::Closed => {
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

    pub fn move_to(&mut self, next: EgressNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress notice {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.weight + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressNotice {
        EgressNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            owner_id: self.owner_id.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a egress notice in status draft.
pub fn new_egress_notice(code: &str, name: &str) -> EgressNotice {
    EgressNotice {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        owner_id: "".to_string(),
        starts_at: 0,
        status: EgressNoticeStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_egress_notice("code-968", "name-661");
        assert_eq!(item.status, EgressNoticeStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_egress_notice("code-968", "name-661");
        if item.move_to(EgressNoticeStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressNoticeStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_egress_notice("code-968", "name-661");
        item.status = EgressNoticeStatus::Closed;
        assert!(
            item.move_to(EgressNoticeStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
