use super::*;

use acme_kit::text;

/// Where a tenant target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantTargetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TenantTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for TenantTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tenant target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantTarget {
    pub code: String,
    pub title: String,
    pub threshold: f64,
    pub unit: String,
    pub segment: String,
    pub host: String,
    pub ratio: f64,
    pub rate: f64,
    pub status: TenantTargetStatus,
}

impl TenantTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantTargetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantTargetStatus) -> bool {
        match self.status {
            TenantTargetStatus::Open => {
                match next {
                    TenantTargetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantTargetStatus::InReview => {
                match next {
                    TenantTargetStatus::Resolved | TenantTargetStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantTargetStatus::Resolved => {
                match next {
                    TenantTargetStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantTargetStatus::Reopened => {
                return next == TenantTargetStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TenantTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        (self.ratio * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantTarget {
        TenantTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            threshold: self.threshold,
            unit: self.unit.clone(),
            segment: self.segment.clone(),
            host: self.host.clone(),
            ratio: self.ratio,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a tenant target in status open.
pub fn new_tenant_target(code: &str, title: &str) -> TenantTarget {
    TenantTarget {
        code: code.to_string(),
        title: title.to_string(),
        threshold: 0.0,
        unit: "".to_string(),
        segment: "".to_string(),
        host: "".to_string(),
        ratio: 0.0,
        rate: 0.0,
        status: TenantTargetStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tenant_target("code-575", "title-249");
        assert_eq!(item.status, TenantTargetStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_tenant_target("code-575", "title-249");
        if item.move_to(TenantTargetStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantTargetStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tenant_target("code-575", "title-249");
        item.status = TenantTargetStatus::Archived;
        assert!(item.move_to(TenantTargetStatus::Open).is_err(), "expected move to fail");
    }
}
