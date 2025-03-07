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
#[macro_use]
extern crate criterion;

use criterion::Criterion;
use std::sync::Arc;

use zenoh::net::protocol::core::Encoding;
use zenoh::net::protocol::core::{Channel, CongestionControl, KeyExpr, PeerId};
use zenoh::net::protocol::io::ZBuf;
use zenoh::net::protocol::proto::{DataInfo, ZenohMessage};

fn consume_message(msg: ZenohMessage) {
    drop(msg);
}

fn consume_message_arc(msg: Arc<ZenohMessage>) {
    drop(msg);
}

fn criterion_benchmark(c: &mut Criterion) {
    let size = [8, 16, 32, 64, 128, 256, 512, 1_024, 2_048, 4_096, 8_192];
    for s in size.iter() {
        c.bench_function(format!("{} msg_creation_yes_info", s).as_str(), |b| {
            b.iter(|| {
                let key_expr = KeyExpr::from(18).with_suffix("/com/acme/sensors/temp");
                let payload = ZBuf::from(vec![0; *s]);
                let channel = Channel::default();
                let congestion_control = CongestionControl::default();
                let info = Some(DataInfo {
                    #[cfg(feature = "shared-memory")]
                    sliced: false,
                    kind: Some(0),
                    encoding: Some(Encoding::default()),
                    timestamp: Some(uhlc::Timestamp::new(
                        Default::default(),
                        uhlc::ID::new(16, [1u8; uhlc::ID::MAX_SIZE]),
                    )),
                    source_id: Some(PeerId::new(16, [0_u8; PeerId::MAX_SIZE])),
                    source_sn: Some(12345),
                    first_router_id: Some(PeerId::new(16, [0_u8; PeerId::MAX_SIZE])),
                    first_router_sn: Some(12345),
                });

                let msg = ZenohMessage::make_data(
                    key_expr,
                    payload,
                    channel,
                    congestion_control,
                    info,
                    None,
                    None,
                    None,
                );
                consume_message(msg);
            })
        });

        c.bench_function(format!("{} msg_creation_no_info", s).as_str(), |b| {
            b.iter(|| {
                let key_expr = KeyExpr::from(18).with_suffix("/com/acme/sensors/temp");
                let payload = ZBuf::from(vec![0; *s]);
                let channel = Channel::default();
                let congestion_control = CongestionControl::default();
                let info = None;

                let msg = ZenohMessage::make_data(
                    key_expr,
                    payload,
                    channel,
                    congestion_control,
                    info,
                    None,
                    None,
                    None,
                );
                consume_message(msg);
            })
        });
    }

    let key_expr = KeyExpr::from(18).with_suffix("/com/acme/sensors/temp");
    let info = Some(DataInfo {
        #[cfg(feature = "shared-memory")]
        sliced: false,
        kind: Some(0),
        encoding: Some(Encoding::default()),
        timestamp: Some(uhlc::Timestamp::new(
            Default::default(),
            uhlc::ID::new(16, [0_u8; uhlc::ID::MAX_SIZE]),
        )),
        source_id: Some(PeerId::new(16, [0_u8; PeerId::MAX_SIZE])),
        source_sn: Some(12345),
        first_router_id: Some(PeerId::new(16, [0_u8; PeerId::MAX_SIZE])),
        first_router_sn: Some(12345),
    });
    let payload = ZBuf::from(vec![0; 1024]);
    let channel = Channel::default();
    let congestion_control = CongestionControl::default();

    let msg = Arc::new(ZenohMessage::make_data(
        key_expr,
        payload,
        channel,
        congestion_control,
        info,
        None,
        None,
        None,
    ));

    c.bench_function("arc_msg_clone", |b| {
        b.iter(|| {
            consume_message_arc(msg.clone());
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
