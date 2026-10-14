use super::*;

use acme_kit::text;

/// Where a lenient action item batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientActionItemBatchStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientActionItemBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for LenientActionItemBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient action item batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemBatch {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub cipher: String,
    pub tags: Vec<String>,
    pub region: String,
    pub status: LenientActionItemBatchStatus,
}

impl LenientActionItemBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientActionItemBatchStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientActionItemBatchStatus) -> bool {
        match self.status {
            LenientActionItemBatchStatus::Idle => {
                return next == LenientActionItemBatchStatus::Warming || next == LenientActionItemBatchStatus::Ready;
            }
            LenientActionItemBatchStatus::Warming => {
                return next == LenientActionItemBatchStatus::Ready;
            }
            LenientActionItemBatchStatus::Ready => {
                match next {
                    LenientActionItemBatchStatus::Draining | LenientActionItemBatchStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientActionItemBatchStatus::Draining => {
                match next {
                    LenientActionItemBatchStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LenientActionItemBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient action item batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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
        self.revision as f64
    }

    pub fn copy(&self) -> LenientActionItemBatch {
        LenientActionItemBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            cipher: self.cipher.clone(),
            tags: self.tags.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a lenient action item batch in status idle.
pub fn new_lenient_action_item_batch(code: &str, label: &str) -> LenientActionItemBatch {
    LenientActionItemBatch {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        cipher: "".to_string(),
        tags: Vec::new(),
        region: "".to_string(),
        status: LenientActionItemBatchStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_lenient_action_item_batch("code-352", "label-728");
        assert_eq!(item.status, LenientActionItemBatchStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_lenient_action_item_batch("code-352", "label-728");
        if item.move_to(LenientActionItemBatchStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientActionItemBatchStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_lenient_action_item_batch("code-352", "label-728");
        item.status = LenientActionItemBatchStatus::Stopped;
        assert!(
            item.move_to(LenientActionItemBatchStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
