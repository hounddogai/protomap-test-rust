use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a lenient slo target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientSloTargetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientSloTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientSloTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient slo target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloTarget {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub limit: i64,
    pub attributes: BTreeMap<String, String>,
    pub status: LenientSloTargetStatus,
}

impl LenientSloTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientSloTargetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientSloTargetStatus) -> bool {
        match self.status {
            LenientSloTargetStatus::Created => {
                return next == LenientSloTargetStatus::Validated || next == LenientSloTargetStatus::Published;
            }
            LenientSloTargetStatus::Validated => {
                match next {
                    LenientSloTargetStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSloTargetStatus::Published => {
                match next {
                    LenientSloTargetStatus::Retired => {
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

    pub fn move_to(&mut self, next: LenientSloTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient slo target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
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

    pub fn effective_volume(&self) -> f64 {
        ((self.margin - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientSloTarget {
        LenientSloTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            limit: self.limit,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient slo target in status created.
pub fn new_lenient_slo_target(code: &str, title: &str) -> LenientSloTarget {
    LenientSloTarget {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        limit: 0,
        attributes: BTreeMap::new(),
        status: LenientSloTargetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_slo_target("code-864", "title-670");
        assert_eq!(item.status, LenientSloTargetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_lenient_slo_target("code-864", "title-670");
        if item.move_to(LenientSloTargetStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientSloTargetStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_slo_target("code-864", "title-670");
        item.status = LenientSloTargetStatus::Retired;
        assert!(
            item.move_to(LenientSloTargetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
