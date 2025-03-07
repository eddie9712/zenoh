//
// Copyright (c) 2022 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//
use crate::StackBuffer;
use async_std::sync::Mutex;
use zenoh_core::zasynclock;
use zenoh_sync::Condition;

pub struct LifoQueue<T> {
    not_empty: Condition,
    not_full: Condition,
    buffer: Mutex<StackBuffer<T>>,
}

impl<T> LifoQueue<T> {
    pub fn new(capacity: usize) -> LifoQueue<T> {
        LifoQueue {
            not_empty: Condition::new(),
            not_full: Condition::new(),
            buffer: Mutex::new(StackBuffer::new(capacity)),
        }
    }

    pub fn try_push(&self, x: T) -> Option<T> {
        if let Some(mut guard) = self.buffer.try_lock() {
            let res = guard.push(x);
            if res.is_none() {
                drop(guard);
                self.not_empty.notify_one();
            }
            return res;
        }
        Some(x)
    }

    pub async fn push(&self, x: T) {
        loop {
            let mut guard = zasynclock!(self.buffer);
            if !guard.is_full() {
                guard.push(x);
                drop(guard);
                self.not_empty.notify_one();
                return;
            }
            self.not_full.wait(guard).await;
        }
    }

    pub fn try_pull(&self) -> Option<T> {
        if let Some(mut guard) = self.buffer.try_lock() {
            if let Some(e) = guard.pop() {
                drop(guard);
                self.not_full.notify_one();
                return Some(e);
            }
        }
        None
    }

    pub async fn pull(&self) -> T {
        loop {
            let mut guard = zasynclock!(self.buffer);
            if let Some(e) = guard.pop() {
                drop(guard);
                self.not_full.notify_one();
                return e;
            }
            self.not_empty.wait(guard).await;
        }
    }
}
