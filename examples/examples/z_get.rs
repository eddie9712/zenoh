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
use clap::{App, Arg};
use futures::prelude::*;
use zenoh::config::Config;
use zenoh::prelude::*;
use zenoh::query::*;
use zenoh::queryable;

#[async_std::main]
async fn main() {
    // initiate logging
    env_logger::init();

    let (config, selector, target) = parse_args();

    println!("Opening session...");
    let session = zenoh::open(config).await.unwrap();

    println!("Sending Query '{}'...", selector);
    let mut replies = session.get(&selector).target(target).await.unwrap();
    while let Some(reply) = replies.next().await {
        println!(
            ">> Received ('{}': '{}')",
            reply.sample.key_expr.as_str(),
            String::from_utf8_lossy(&reply.sample.value.payload.contiguous())
        )
    }
}

fn parse_args() -> (Config, String, QueryTarget) {
    let args = App::new("zenoh query example")
        .arg(
            Arg::from_usage("-m, --mode=[MODE]  'The zenoh session mode (peer by default).")
                .possible_values(&["peer", "client"]),
        )
        .arg(Arg::from_usage(
            "-e, --connect=[ENDPOINT]...   'Endpoints to connect to.'",
        ))
        .arg(Arg::from_usage(
            "-l, --listen=[ENDPOINT]...   'Endpoints to listen on.'",
        ))
        .arg(
            Arg::from_usage("-s, --selector=[SELECTOR] 'The selection of resources to query'")
                .default_value("/demo/example/**"),
        )
        .arg(
            Arg::from_usage("-k, --kind=[KIND] 'The KIND of queryables to query'")
                .possible_values(&["ALL_KINDS", "STORAGE", "EVAL"])
                .default_value("ALL_KINDS"),
        )
        .arg(
            Arg::from_usage("-t, --target=[TARGET] 'The target queryables of the query'")
                .possible_values(&["ALL", "BEST_MATCHING", "ALL_COMPLETE", "NONE"])
                .default_value("ALL"),
        )
        .arg(Arg::from_usage(
            "-c, --config=[FILE]      'A configuration file.'",
        ))
        .arg(Arg::from_usage(
            "--no-multicast-scouting 'Disable the multicast-based scouting mechanism.'",
        ))
        .get_matches();

    let mut config = if let Some(conf_file) = args.value_of("config") {
        Config::from_file(conf_file).unwrap()
    } else {
        Config::default()
    };
    if let Some(Ok(mode)) = args.value_of("mode").map(|mode| mode.parse()) {
        config.set_mode(Some(mode)).unwrap();
    }
    if let Some(values) = args.values_of("connect") {
        config
            .connect
            .endpoints
            .extend(values.map(|v| v.parse().unwrap()))
    }
    if let Some(values) = args.values_of("listen") {
        config
            .listen
            .endpoints
            .extend(values.map(|v| v.parse().unwrap()))
    }
    if args.is_present("no-multicast-scouting") {
        config.scouting.multicast.set_enabled(Some(false)).unwrap();
    }

    let selector = args.value_of("selector").unwrap().to_string();

    let kind = match args.value_of("kind") {
        Some("STORAGE") => queryable::STORAGE,
        Some("EVAL") => queryable::EVAL,
        _ => queryable::ALL_KINDS,
    };

    let target = match args.value_of("target") {
        Some("BEST_MATCHING") => Target::BestMatching,
        Some("ALL_COMPLETE") => Target::AllComplete,
        Some("NONE") => Target::None,
        _ => Target::All,
    };

    (config, selector, QueryTarget { kind, target })
}
