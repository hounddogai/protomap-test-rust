use super::*;

use acme_kit::text;

/// Where a tiered token segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredTokenSegmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredTokenSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredTokenSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered token segment as the token team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenSegment {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub tags: Vec<String>,
    pub source: String,
    pub grace_days: i64,
    pub status: TieredTokenSegmentStatus,
}

impl TieredTokenSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredTokenSegmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredTokenSegmentStatus) -> bool {
        match self.status {
            TieredTokenSegmentStatus::Created => {
                return next == TieredTokenSegmentStatus::Validated;
            }
            TieredTokenSegmentStatus::Validated => {
                return next == TieredTokenSegmentStatus::Published || next == TieredTokenSegmentStatus::Retired;
            }
            TieredTokenSegmentStatus::Published => {
                return next == TieredTokenSegmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredTokenSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered token segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.grace_days as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredTokenSegment {
        TieredTokenSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            tags: self.tags.clone(),
            source: self.source.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a tiered token segment in status created.
pub fn new_tiered_token_segment(code: &str, name: &str) -> TieredTokenSegment {
    TieredTokenSegment {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        tags: Vec::new(),
        source: "".to_string(),
        grace_days: 0,
        status: TieredTokenSegmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tiered_token_segment("code-780", "name-299");
        assert_eq!(item.status, TieredTokenSegmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_tiered_token_segment("code-780", "name-299");
        if item.move_to(TieredTokenSegmentStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredTokenSegmentStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tiered_token_segment("code-780", "name-299");
        item.status = TieredTokenSegmentStatus::Retired;
        assert!(
            item.move_to(TieredTokenSegmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
