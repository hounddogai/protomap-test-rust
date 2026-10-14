use super::*;

use acme_kit::text;

/// Where a tenant claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantClaimStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TenantClaimStatus {
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

impl std::fmt::Display for TenantClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tenant claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantClaim {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub region: String,
    pub notes: String,
    pub tags: Vec<String>,
    pub sequence: i64,
    pub status: TenantClaimStatus,
}

impl TenantClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantClaimStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantClaimStatus) -> bool {
        match self.status {
            TenantClaimStatus::New => {
                return next == TenantClaimStatus::Running;
            }
            TenantClaimStatus::Queued => {
                return next == TenantClaimStatus::Failed;
            }
            TenantClaimStatus::Running => {
                return next == TenantClaimStatus::Failed;
            }
            TenantClaimStatus::Done => {
                match next {
                    TenantClaimStatus::Failed => {
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

    pub fn move_to(&mut self, next: TenantClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tenant claim", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.margin - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantClaim {
        TenantClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            region: self.region.clone(),
            notes: self.notes.clone(),
            tags: self.tags.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a tenant claim in status new.
pub fn new_tenant_claim(code: &str, title: &str) -> TenantClaim {
    TenantClaim {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        region: "".to_string(),
        notes: "".to_string(),
        tags: Vec::new(),
        sequence: 0,
        status: TenantClaimStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_tenant_claim("code-764", "title-167");
        assert_eq!(item.status, TenantClaimStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_tenant_claim("code-764", "title-167");
        if item.move_to(TenantClaimStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantClaimStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_tenant_claim("code-764", "title-167");
        item.status = TenantClaimStatus::Failed;
        assert!(item.move_to(TenantClaimStatus::New).is_err(), "expected move to fail");
    }
}
