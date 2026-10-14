use super::*;

use acme_kit::text;

/// Where a global retention claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalRetentionClaimStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalRetentionClaimStatus {
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

impl std::fmt::Display for GlobalRetentionClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global retention claim as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRetentionClaim {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub description: String,
    pub issuer: String,
    pub expires_at: i64,
    pub audience: String,
    pub status: GlobalRetentionClaimStatus,
}

impl GlobalRetentionClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalRetentionClaimStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalRetentionClaimStatus) -> bool {
        match self.status {
            GlobalRetentionClaimStatus::Draft => {
                return next == GlobalRetentionClaimStatus::Closed;
            }
            GlobalRetentionClaimStatus::Pending => {
                match next {
                    GlobalRetentionClaimStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalRetentionClaimStatus::Active => {
                return next == GlobalRetentionClaimStatus::Closed;
            }
            GlobalRetentionClaimStatus::Suspended => {
                return next == GlobalRetentionClaimStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalRetentionClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global retention claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "global retention claim", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.expires_at as f64
    }

    pub fn copy(&self) -> GlobalRetentionClaim {
        GlobalRetentionClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            description: self.description.clone(),
            issuer: self.issuer.clone(),
            expires_at: self.expires_at,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global retention claim in status draft.
pub fn new_global_retention_claim(code: &str, label: &str) -> GlobalRetentionClaim {
    GlobalRetentionClaim {
        code: code.to_string(),
        label: label.to_string(),
        phone: "".to_string(),
        description: "".to_string(),
        issuer: "".to_string(),
        expires_at: 0,
        audience: "".to_string(),
        status: GlobalRetentionClaimStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_global_retention_claim("code-300", "label-75");
        assert_eq!(item.status, GlobalRetentionClaimStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_global_retention_claim("code-300", "label-75");
        if item.move_to(GlobalRetentionClaimStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalRetentionClaimStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_global_retention_claim("code-300", "label-75");
        item.status = GlobalRetentionClaimStatus::Closed;
        assert!(
            item.move_to(GlobalRetentionClaimStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
