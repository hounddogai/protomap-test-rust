use super::*;

use acme_kit::text;

/// Where a adaptive measurement entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveMeasurementEntryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AdaptiveMeasurementEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AdaptiveMeasurementEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive measurement entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementEntry {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub tags: Vec<String>,
    pub source: String,
    pub checksum: String,
    pub host: String,
    pub status: AdaptiveMeasurementEntryStatus,
}

impl AdaptiveMeasurementEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveMeasurementEntryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveMeasurementEntryStatus) -> bool {
        match self.status {
            AdaptiveMeasurementEntryStatus::Created => {
                match next {
                    AdaptiveMeasurementEntryStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveMeasurementEntryStatus::Validated => {
                match next {
                    AdaptiveMeasurementEntryStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveMeasurementEntryStatus::Published => {
                return next == AdaptiveMeasurementEntryStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveMeasurementEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive measurement entry {} cannot move from {} to {}",
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

    pub fn measured_value(&self) -> f64 {
        self.sequence as f64
    }

    pub fn copy(&self) -> AdaptiveMeasurementEntry {
        AdaptiveMeasurementEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            tags: self.tags.clone(),
            source: self.source.clone(),
            checksum: self.checksum.clone(),
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a adaptive measurement entry in status created.
pub fn new_adaptive_measurement_entry(code: &str, name: &str) -> AdaptiveMeasurementEntry {
    AdaptiveMeasurementEntry {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        tags: Vec::new(),
        source: "".to_string(),
        checksum: "".to_string(),
        host: "".to_string(),
        status: AdaptiveMeasurementEntryStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_adaptive_measurement_entry("code-541", "name-910");
        assert_eq!(item.status, AdaptiveMeasurementEntryStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_adaptive_measurement_entry("code-541", "name-910");
        if item.move_to(AdaptiveMeasurementEntryStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveMeasurementEntryStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_adaptive_measurement_entry("code-541", "name-910");
        item.status = AdaptiveMeasurementEntryStatus::Retired;
        assert!(
            item.move_to(AdaptiveMeasurementEntryStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
