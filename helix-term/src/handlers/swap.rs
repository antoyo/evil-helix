// TODO: remove the swap file on save (only when it's correctly synced on disk).
//
// TODO: this mechanism could also be used to detect attempts to edit a file
// that is already being edited.

use std::time::Duration;

use anyhow::Ok;
use arc_swap::access::Access;

use helix_event::{register_hook, send_blocking};
use helix_view::{
    events::DocumentDidChange,
    handlers::{Handlers, SwapEvent},
    Editor,
};
use tokio::time::Instant;

use crate::{
    commands, compositor,
    job::{self, Jobs},
};

#[derive(Debug)]
pub(super) struct SwapHandler {
}

impl SwapHandler {
    pub fn new() -> SwapHandler {
        SwapHandler {
        }
    }
}

impl helix_event::AsyncHook for SwapHandler {
    type Event = SwapEvent;

    fn handle_event(
        &mut self,
        event: Self::Event,
        _existing_debounce: Option<tokio::time::Instant>,
    ) -> Option<Instant> {
        match event {
            Self::Event::DocumentChanged { save_after } => {
                Some(Instant::now() + Duration::from_millis(save_after))
            }
        }
    }

    fn finish_debounce(&mut self) {
        job::dispatch_blocking(move |editor, _| {
            request_swap(editor);
        })
    }
}

fn request_swap(editor: &mut Editor) {
    let context = &mut compositor::Context {
        editor,
        scroll: Some(0),
        jobs: &mut Jobs::new(),
    };

    if let Err(e) = commands::evil::write_all_swap(context) {
        context.editor.set_error(format!("{}", e));
    }
}

pub(super) fn register_hooks(handlers: &Handlers) {
    let tx = handlers.swap.clone();
    register_hook!(move |event: &mut DocumentDidChange<'_>| {
        let config = event.doc.config.load();
        // TODO: add settings for swap and use them instead of auto_save settings.
        if config.auto_save.after_delay.enable {
            send_blocking(
                &tx,
                SwapEvent::DocumentChanged {
                    save_after: config.auto_save.after_delay.timeout,
                },
            );
        }
        Ok(())
    });
}
