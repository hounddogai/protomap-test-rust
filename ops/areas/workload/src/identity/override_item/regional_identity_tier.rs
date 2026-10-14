use super::*;

use acme_kit::text;

/// Where a regional identity tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalIdentityTierStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionalIdentityTierStatus {
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

impl std::fmt::Display for RegionalIdentityTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional identity tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalIdentityTier {
    pub code: String,
    pub title: String,
    pub cost: f64,
    pub tags: Vec<String>,
    pub host: String,
    pub limit: i64,
    pub version: i64,
    pub audience: String,
    pub rate: f64,
    pub rotation_days: i64,
    pub quantity: i64,
    pub status: RegionalIdentityTierStatus,
}

impl RegionalIdentityTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalIdentityTierStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalIdentityTierStatus) -> bool {
        match self.status {
            RegionalIdentityTierStatus::New => {
                match next {
                    RegionalIdentityTierStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalIdentityTierStatus::Queued => {
                return next == RegionalIdentityTierStatus::Running || next == RegionalIdentityTierStatus::Failed;
            }
            RegionalIdentityTierStatus::Running => {
                return next == RegionalIdentityTierStatus::Done;
            }
            RegionalIdentityTierStatus::Done => {
                return next == RegionalIdentityTierStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalIdentityTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional identity tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional identity tier", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.rotation_days as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalIdentityTier {
        RegionalIdentityTier {
            code: self.code.clone(),
            title: self.title.clone(),
            cost: self.cost,
            tags: self.tags.clone(),
            host: self.host.clone(),
            limit: self.limit,
            version: self.version,
            audience: self.audience.clone(),
            rate: self.rate,
            rotation_days: self.rotation_days,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional identity tier in status new.
pub fn new_regional_identity_tier(code: &str, title: &str) -> RegionalIdentityTier {
    RegionalIdentityTier {
        code: code.to_string(),
        title: title.to_string(),
        cost: 0.0,
        tags: Vec::new(),
        host: "".to_string(),
        limit: 0,
        version: 0,
        audience: "".to_string(),
        rate: 0.0,
        rotation_days: 0,
        quantity: 0,
        status: RegionalIdentityTierStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_regional_identity_tier("code-442", "title-842");
        assert_eq!(item.status, RegionalIdentityTierStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_regional_identity_tier("code-442", "title-842");
        if item.move_to(RegionalIdentityTierStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalIdentityTierStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_regional_identity_tier("code-442", "title-842");
        item.status = RegionalIdentityTierStatus::Failed;
        assert!(
            item.move_to(RegionalIdentityTierStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
