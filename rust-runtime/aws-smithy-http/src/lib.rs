/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![warn(
    missing_docs,
    rustdoc::missing_crate_level_docs,
    unreachable_pub,
    rust_2018_idioms
)]

//! Core HTTP primitives for service clients generated by [smithy-rs](https://github.com/awslabs/smithy-rs) including:
//! - HTTP Body implementation
//! - Endpoint support
//! - HTTP header deserialization
//! - Event streams
//! - [`ByteStream`](byte_stream::ByteStream): a misuse-resistant abstraction for streaming binary data
//!
//! | Feature        | Description |
//! |----------------|-------------|
//! | `rt-tokio`     | Provides features that are dependent on `tokio` including the `ByteStream::from_path` util |
//! | `event-stream` | Provides Sender/Receiver implementations for Event Stream codegen. |

#![allow(clippy::derive_partial_eq_without_eq)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod body;
pub mod endpoint;
// Marked as `doc(hidden)` because a type in the module is used both by this crate and by the code
// generator, but not by external users. Also, by the module being `doc(hidden)` instead of it being
// in `rust-runtime/inlineable`, each user won't have to pay the cost of running the module's tests
// when compiling their generated SDK.
#[doc(hidden)]
pub mod futures_stream_adapter;
pub mod header;
pub mod http;
pub mod label;
pub mod middleware;
pub mod operation;
pub mod property_bag;
pub mod query;
#[doc(hidden)]
pub mod query_writer;
pub mod response;
pub mod result;
pub mod retry;

#[cfg(feature = "event-stream")]
pub mod event_stream;

pub mod byte_stream;

pub mod connection;
mod urlencode;
