use super::*;

use acme_kit::text;

/// Where a legacy silence allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacySilenceAllocationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacySilenceAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacySilenceAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy silence allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceAllocation {
    pub code: String,
    pub title: String,
    pub threshold: f64,
    pub ttl_seconds: i64,
    pub revision: i64,
    pub country: String,
    pub status: LegacySilenceAllocationStatus,
}

impl LegacySilenceAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacySilenceAllocationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacySilenceAllocationStatus) -> bool {
        match self.status {
            LegacySilenceAllocationStatus::Created => {
                match next {
                    LegacySilenceAllocationStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacySilenceAllocationStatus::Validated => {
                return next == LegacySilenceAllocationStatus::Published
                    || next == LegacySilenceAllocationStatus::Retired;
            }
            LegacySilenceAllocationStatus::Published => {
                match next {
                    LegacySilenceAllocationStatus::Retired => {
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

    pub fn move_to(&mut self, next: LegacySilenceAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy silence allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy silence allocation", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.revision as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacySilenceAllocation {
        LegacySilenceAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            threshold: self.threshold,
            ttl_seconds: self.ttl_seconds,
            revision: self.revision,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a legacy silence allocation in status created.
pub fn new_legacy_silence_allocation(code: &str, title: &str) -> LegacySilenceAllocation {
    LegacySilenceAllocation {
        code: code.to_string(),
        title: title.to_string(),
        threshold: 0.0,
        ttl_seconds: 0,
        revision: 0,
        country: "".to_string(),
        status: LegacySilenceAllocationStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_legacy_silence_allocation("code-541", "title-153");
        assert_eq!(item.status, LegacySilenceAllocationStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_legacy_silence_allocation("code-541", "title-153");
        if item.move_to(LegacySilenceAllocationStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacySilenceAllocationStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_legacy_silence_allocation("code-541", "title-153");
        item.status = LegacySilenceAllocationStatus::Retired;
        assert!(
            item.move_to(LegacySilenceAllocationStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
