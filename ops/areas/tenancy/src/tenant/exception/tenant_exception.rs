use super::*;

use acme_kit::text;

/// Where a tenant exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantExceptionStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TenantExceptionStatus {
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

impl std::fmt::Display for TenantExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tenant exception, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantException {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub segment: String,
    pub cipher: String,
    pub updated_at: i64,
    pub archived: bool,
    pub phone: String,
    pub timezone: String,
    pub status: TenantExceptionStatus,
}

impl TenantException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantExceptionStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantExceptionStatus) -> bool {
        match self.status {
            TenantExceptionStatus::Idle => {
                match next {
                    TenantExceptionStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantExceptionStatus::Warming => {
                match next {
                    TenantExceptionStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantExceptionStatus::Ready => {
                match next {
                    TenantExceptionStatus::Draining | TenantExceptionStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantExceptionStatus::Draining => {
                return next == TenantExceptionStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TenantExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant exception {} cannot move from {} to {}",
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
        format!("{} {}", "tenant exception", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.updated_at as f64
    }

    pub fn copy(&self) -> TenantException {
        TenantException {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            segment: self.segment.clone(),
            cipher: self.cipher.clone(),
            updated_at: self.updated_at,
            archived: self.archived,
            phone: self.phone.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a tenant exception in status idle.
pub fn new_tenant_exception(code: &str, name: &str) -> TenantException {
    TenantException {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        segment: "".to_string(),
        cipher: "".to_string(),
        updated_at: 0,
        archived: false,
        phone: "".to_string(),
        timezone: "".to_string(),
        status: TenantExceptionStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tenant_exception("code-596", "name-877");
        assert_eq!(item.status, TenantExceptionStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_tenant_exception("code-596", "name-877");
        if item.move_to(TenantExceptionStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantExceptionStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tenant_exception("code-596", "name-877");
        item.status = TenantExceptionStatus::Stopped;
        assert!(
            item.move_to(TenantExceptionStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
