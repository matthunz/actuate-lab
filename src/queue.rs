use actuate::prelude::*;
use std::{cell::Cell, future::Future};
use tokio::sync::mpsc;

struct Queue {
    tx: mpsc::UnboundedSender<mpsc::UnboundedSender<()>>,
}

pub fn use_queue_provider(cx: ScopeState) {
    let (tx, rx) = use_ref(cx, || {
        let (tx, rx) = mpsc::unbounded_channel::<mpsc::UnboundedSender<()>>();
        (tx, Cell::new(Some(rx)))
    });

    use_local_task(cx, move || async move {
        let mut rx = rx.take().unwrap();
        while let Some(tx) = rx.recv().await {
            tx.send(()).unwrap();
        }
    });

    use_provider(cx, || Queue { tx: tx.clone() });
}

pub fn use_queued<'a, F: Future>(
    cx: ScopeState<'a>,
    mut make_future: impl FnMut() -> F + 'a,
) -> UseQueued<'a> {
    let queue = use_context::<Queue>(cx).unwrap();
    let (tx, rx) = use_ref(cx, || {
        let (tx, rx) = mpsc::unbounded_channel();
        (tx, Cell::new(Some(rx)))
    });

    use_local_task(cx, move || async move {
        let mut rx = rx.take().unwrap();
        while let Some(()) = rx.recv().await {
            make_future().await;
        }
    });

    UseQueued { queue, tx }
}

#[derive(Clone, Copy)]
pub struct UseQueued<'a> {
    queue: &'a Queue,
    tx: &'a mpsc::UnboundedSender<()>,
}

impl UseQueued<'_> {
    pub fn queue(&self) {
        self.queue.tx.send(self.tx.clone()).unwrap();
    }
}
