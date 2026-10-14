use super::*;

use acme_kit::text;

/// Where a adaptive slo record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveSloRecordStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AdaptiveSloRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AdaptiveSloRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive slo record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloRecord {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub grace_days: i64,
    pub tags: Vec<String>,
    pub timezone: String,
    pub archived: bool,
    pub status: AdaptiveSloRecordStatus,
}

impl AdaptiveSloRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveSloRecordStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveSloRecordStatus) -> bool {
        match self.status {
            AdaptiveSloRecordStatus::Created => {
                match next {
                    AdaptiveSloRecordStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveSloRecordStatus::Validated => {
                match next {
                    AdaptiveSloRecordStatus::Published | AdaptiveSloRecordStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveSloRecordStatus::Published => {
                return next == AdaptiveSloRecordStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveSloRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive slo record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
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
        self.grace_days as f64
    }

    pub fn copy(&self) -> AdaptiveSloRecord {
        AdaptiveSloRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            grace_days: self.grace_days,
            tags: self.tags.clone(),
            timezone: self.timezone.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a adaptive slo record in status created.
pub fn new_adaptive_slo_record(code: &str, name: &str) -> AdaptiveSloRecord {
    AdaptiveSloRecord {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        grace_days: 0,
        tags: Vec::new(),
        timezone: "".to_string(),
        archived: false,
        status: AdaptiveSloRecordStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_adaptive_slo_record("code-954", "name-229");
        assert_eq!(item.status, AdaptiveSloRecordStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_adaptive_slo_record("code-954", "name-229");
        if item.move_to(AdaptiveSloRecordStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveSloRecordStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_adaptive_slo_record("code-954", "name-229");
        item.status = AdaptiveSloRecordStatus::Retired;
        assert!(
            item.move_to(AdaptiveSloRecordStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
