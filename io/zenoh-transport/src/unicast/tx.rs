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
#[cfg(feature = "stats")]
use super::protocol::proto::ZenohBody;
use super::protocol::proto::ZenohMessage;
use super::transport::TransportUnicastInner;
#[cfg(feature = "stats")]
use zenoh_buffers::SplitBuffer;
use zenoh_core::zread;

impl TransportUnicastInner {
    fn schedule_on_link(&self, msg: ZenohMessage) -> bool {
        macro_rules! zpush {
            ($guard:expr, $pipeline:expr, $msg:expr) => {
                // Drop the guard before the push_zenoh_message since
                // the link could be congested and this operation could
                // block for fairly long time
                let pl = $pipeline.clone();
                drop($guard);
                return pl.push_zenoh_message($msg);
            };
        }

        let guard = zread!(self.links);
        // First try to find the best match between msg and link reliability
        for tl in guard.iter() {
            if let Some(pipeline) = tl.pipeline.as_ref() {
                if msg.is_reliable() && tl.link.is_reliable() {
                    zpush!(guard, pipeline, msg);
                }
                if !msg.is_reliable() && !tl.link.is_reliable() {
                    zpush!(guard, pipeline, msg);
                }
            }
        }

        // No best match found, take the first available link
        for tl in guard.iter() {
            if let Some(pipeline) = tl.pipeline.as_ref() {
                zpush!(guard, pipeline, msg);
            }
        }

        // No Link found
        log::trace!(
            "Message dropped because the transport has no links: {}",
            msg
        );

        false
    }

    #[allow(clippy::let_and_return)] // When feature "stats" is not enabled
    #[inline(always)]
    pub(super) fn schedule_first_fit(&self, msg: ZenohMessage) -> bool {
        #[cfg(feature = "stats")]
        match &msg.body {
            ZenohBody::Data(data) => match data.reply_context {
                Some(_) => {
                    self.stats.inc_tx_z_data_reply_msgs(1);
                    self.stats
                        .inc_tx_z_data_reply_payload_bytes(data.payload.len());
                }
                None => {
                    self.stats.inc_tx_z_data_msgs(1);
                    self.stats.inc_tx_z_data_payload_bytes(data.payload.len());
                }
            },
            ZenohBody::Unit(unit) => match unit.reply_context {
                Some(_) => self.stats.inc_tx_z_unit_reply_msgs(1),
                None => self.stats.inc_tx_z_unit_msgs(1),
            },
            ZenohBody::Pull(_) => self.stats.inc_tx_z_pull_msgs(1),
            ZenohBody::Query(_) => self.stats.inc_tx_z_query_msgs(1),
            ZenohBody::Declare(_) => self.stats.inc_tx_z_declare_msgs(1),
            ZenohBody::LinkStateList(_) => self.stats.inc_tx_z_linkstate_msgs(1),
        }

        let res = self.schedule_on_link(msg);

        #[cfg(feature = "stats")]
        if res {
            self.stats.inc_tx_z_msgs(1);
        } else {
            self.stats.inc_tx_z_dropped(1);
        }

        res
    }
}
