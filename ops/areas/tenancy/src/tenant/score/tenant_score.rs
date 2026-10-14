use super::*;

use acme_kit::text;

/// Where a tenant score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantScoreStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TenantScoreStatus {
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

impl std::fmt::Display for TenantScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tenant score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantScore {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub tags: Vec<String>,
    pub country: String,
    pub name: String,
    pub status: TenantScoreStatus,
}

impl TenantScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantScoreStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantScoreStatus) -> bool {
        match self.status {
            TenantScoreStatus::Idle => {
                match next {
                    TenantScoreStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantScoreStatus::Warming => {
                return next == TenantScoreStatus::Ready || next == TenantScoreStatus::Draining;
            }
            TenantScoreStatus::Ready => {
                match next {
                    TenantScoreStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantScoreStatus::Draining => {
                return next == TenantScoreStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TenantScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tenant score", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> TenantScore {
        TenantScore {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            tags: self.tags.clone(),
            country: self.country.clone(),
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a tenant score in status idle.
pub fn new_tenant_score(code: &str, title: &str) -> TenantScore {
    TenantScore {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        tags: Vec::new(),
        country: "".to_string(),
        name: "".to_string(),
        status: TenantScoreStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tenant_score("code-679", "title-612");
        assert_eq!(item.status, TenantScoreStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_tenant_score("code-679", "title-612");
        if item.move_to(TenantScoreStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantScoreStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tenant_score("code-679", "title-612");
        item.status = TenantScoreStatus::Stopped;
        assert!(item.move_to(TenantScoreStatus::Idle).is_err(), "expected move to fail");
    }
}
