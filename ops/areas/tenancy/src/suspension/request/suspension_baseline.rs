use super::*;

use acme_kit::text;

/// Where a suspension baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionBaselineStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SuspensionBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SuspensionBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a suspension baseline and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionBaseline {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub source: String,
    pub segment: String,
    pub status: SuspensionBaselineStatus,
}

impl SuspensionBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionBaselineStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionBaselineStatus) -> bool {
        match self.status {
            SuspensionBaselineStatus::Created => {
                match next {
                    SuspensionBaselineStatus::Published | SuspensionBaselineStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionBaselineStatus::Validated => {
                return next == SuspensionBaselineStatus::Retired;
            }
            SuspensionBaselineStatus::Published => {
                match next {
                    SuspensionBaselineStatus::Retired => {
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

    pub fn move_to(&mut self, next: SuspensionBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.updated_at as f64
    }

    pub fn copy(&self) -> SuspensionBaseline {
        SuspensionBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            source: self.source.clone(),
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a suspension baseline in status created.
pub fn new_suspension_baseline(code: &str, name: &str) -> SuspensionBaseline {
    SuspensionBaseline {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        source: "".to_string(),
        segment: "".to_string(),
        status: SuspensionBaselineStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_suspension_baseline("code-535", "name-750");
        assert_eq!(item.status, SuspensionBaselineStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_suspension_baseline("code-535", "name-750");
        if item.move_to(SuspensionBaselineStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionBaselineStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_suspension_baseline("code-535", "name-750");
        item.status = SuspensionBaselineStatus::Retired;
        assert!(
            item.move_to(SuspensionBaselineStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
