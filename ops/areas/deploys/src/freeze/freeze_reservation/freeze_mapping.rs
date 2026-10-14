use super::*;

use acme_kit::text;

/// Where a freeze mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeMappingStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FreezeMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FreezeMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeMapping {
    pub code: String,
    pub title: String,
    pub host: String,
    pub ratio: f64,
    pub margin: f64,
    pub grace_days: i64,
    pub phone: String,
    pub checksum: String,
    pub burst: i64,
    pub status: FreezeMappingStatus,
}

impl FreezeMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeMappingStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeMappingStatus) -> bool {
        match self.status {
            FreezeMappingStatus::Created => {
                match next {
                    FreezeMappingStatus::Validated | FreezeMappingStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeMappingStatus::Validated => {
                return next == FreezeMappingStatus::Published || next == FreezeMappingStatus::Retired;
            }
            FreezeMappingStatus::Published => {
                match next {
                    FreezeMappingStatus::Retired => {
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

    pub fn move_to(&mut self, next: FreezeMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "freeze mapping", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.ratio - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeMapping {
        FreezeMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            ratio: self.ratio,
            margin: self.margin,
            grace_days: self.grace_days,
            phone: self.phone.clone(),
            checksum: self.checksum.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a freeze mapping in status created.
pub fn new_freeze_mapping(code: &str, title: &str) -> FreezeMapping {
    FreezeMapping {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        ratio: 0.0,
        margin: 0.0,
        grace_days: 0,
        phone: "".to_string(),
        checksum: "".to_string(),
        burst: 0,
        status: FreezeMappingStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_freeze_mapping("code-68", "title-847");
        assert_eq!(item.status, FreezeMappingStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_freeze_mapping("code-68", "title-847");
        if item.move_to(FreezeMappingStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeMappingStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_freeze_mapping("code-68", "title-847");
        item.status = FreezeMappingStatus::Retired;
        assert!(
            item.move_to(FreezeMappingStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
