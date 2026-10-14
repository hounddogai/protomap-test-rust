use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves fallback segment drafts through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct FallbackSegmentDraftWorkflow {
    pub store: Rc<dyn FallbackSegmentDraftStore>,
    pub validator: FallbackSegmentDraftValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl FallbackSegmentDraftWorkflow {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback segment draft {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(FallbackSegmentDraftStatus::Validated)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is validated", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("fallback segment draft {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(FallbackSegmentDraftStatus::Retired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is retired: {}", code, reason));
        Ok(())
    }

    pub fn scan(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            let problems = self.validator.validate(&item);
            if (problems.len() as i64) == 0 {
                continue;
            }
            self.note(&format!("{} needs attention: {}", item.code, problems.join(", ")));
            changed += 1;
        }
        changed
    }
}

pub fn new_fallback_segment_draft_workflow(
    store: Rc<dyn FallbackSegmentDraftStore>,
    clock: Rc<dyn clock::Clock>,
) -> FallbackSegmentDraftWorkflow {
    FallbackSegmentDraftWorkflow {
        store: store.clone(),
        validator: default_fallback_segment_draft_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
