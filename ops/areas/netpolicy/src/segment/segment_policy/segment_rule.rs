use super::*;

use acme_kit::text;

/// Where a segment rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentRuleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SegmentRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SegmentRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment rule as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRule {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub minimum: f64,
    pub checksum: String,
    pub rotation_days: i64,
    pub ends_at: i64,
    pub cipher: String,
    pub status: SegmentRuleStatus,
}

impl SegmentRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentRuleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentRuleStatus) -> bool {
        match self.status {
            SegmentRuleStatus::Created => {
                match next {
                    SegmentRuleStatus::Validated | SegmentRuleStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentRuleStatus::Validated => {
                match next {
                    SegmentRuleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentRuleStatus::Published => {
                match next {
                    SegmentRuleStatus::Retired => {
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

    pub fn move_to(&mut self, next: SegmentRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment rule {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "segment rule", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.minimum * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentRule {
        SegmentRule {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            minimum: self.minimum,
            checksum: self.checksum.clone(),
            rotation_days: self.rotation_days,
            ends_at: self.ends_at,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a segment rule in status created.
pub fn new_segment_rule(code: &str, label: &str) -> SegmentRule {
    SegmentRule {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        minimum: 0.0,
        checksum: "".to_string(),
        rotation_days: 0,
        ends_at: 0,
        cipher: "".to_string(),
        status: SegmentRuleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_segment_rule("code-82", "label-570");
        assert_eq!(item.status, SegmentRuleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_segment_rule("code-82", "label-570");
        if item.move_to(SegmentRuleStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentRuleStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_segment_rule("code-82", "label-570");
        item.status = SegmentRuleStatus::Retired;
        assert!(
            item.move_to(SegmentRuleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
