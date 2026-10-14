use super::*;

use acme_kit::text;

/// Where a lenient migration profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientMigrationProfileStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientMigrationProfileStatus {
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

impl std::fmt::Display for LenientMigrationProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient migration profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationProfile {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub capacity: i64,
    pub ends_at: i64,
    pub starts_at: i64,
    pub threshold: f64,
    pub ttl_seconds: i64,
    pub limit: i64,
    pub retries: i64,
    pub cluster: String,
    pub status: LenientMigrationProfileStatus,
}

impl LenientMigrationProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientMigrationProfileStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientMigrationProfileStatus) -> bool {
        match self.status {
            LenientMigrationProfileStatus::Draft => {
                return next == LenientMigrationProfileStatus::Active || next == LenientMigrationProfileStatus::Closed;
            }
            LenientMigrationProfileStatus::Pending => {
                match next {
                    LenientMigrationProfileStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientMigrationProfileStatus::Active => {
                return next == LenientMigrationProfileStatus::Suspended
                    || next == LenientMigrationProfileStatus::Closed;
            }
            LenientMigrationProfileStatus::Suspended => {
                return next == LenientMigrationProfileStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientMigrationProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient migration profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.ends_at as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientMigrationProfile {
        LenientMigrationProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            capacity: self.capacity,
            ends_at: self.ends_at,
            starts_at: self.starts_at,
            threshold: self.threshold,
            ttl_seconds: self.ttl_seconds,
            limit: self.limit,
            retries: self.retries,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a lenient migration profile in status draft.
pub fn new_lenient_migration_profile(code: &str, title: &str) -> LenientMigrationProfile {
    LenientMigrationProfile {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        capacity: 0,
        ends_at: 0,
        starts_at: 0,
        threshold: 0.0,
        ttl_seconds: 0,
        limit: 0,
        retries: 0,
        cluster: "".to_string(),
        status: LenientMigrationProfileStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_lenient_migration_profile("code-847", "title-420");
        assert_eq!(item.status, LenientMigrationProfileStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_lenient_migration_profile("code-847", "title-420");
        if item.move_to(LenientMigrationProfileStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientMigrationProfileStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_lenient_migration_profile("code-847", "title-420");
        item.status = LenientMigrationProfileStatus::Closed;
        assert!(
            item.move_to(LenientMigrationProfileStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
