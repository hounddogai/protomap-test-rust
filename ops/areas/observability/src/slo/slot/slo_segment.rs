use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a slo segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloSegmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SloSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SloSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo segment as the slo team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloSegment {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub fingerprint: String,
    pub starts_at: i64,
    pub status: SloSegmentStatus,
}

impl SloSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloSegmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloSegmentStatus) -> bool {
        match self.status {
            SloSegmentStatus::Created => {
                return next == SloSegmentStatus::Published || next == SloSegmentStatus::Retired;
            }
            SloSegmentStatus::Validated => {
                match next {
                    SloSegmentStatus::Published | SloSegmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloSegmentStatus::Published => {
                match next {
                    SloSegmentStatus::Retired => {
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

    pub fn move_to(&mut self, next: SloSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.starts_at as f64
    }

    pub fn copy(&self) -> SloSegment {
        SloSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            fingerprint: self.fingerprint.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a slo segment in status created.
pub fn new_slo_segment(code: &str, title: &str) -> SloSegment {
    SloSegment {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        fingerprint: "".to_string(),
        starts_at: 0,
        status: SloSegmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_slo_segment("code-97", "title-219");
        assert_eq!(item.status, SloSegmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_slo_segment("code-97", "title-219");
        if item.move_to(SloSegmentStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloSegmentStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_slo_segment("code-97", "title-219");
        item.status = SloSegmentStatus::Retired;
        assert!(
            item.move_to(SloSegmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
