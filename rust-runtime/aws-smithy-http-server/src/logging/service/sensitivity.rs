/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
use std::fmt;

use http::header::HeaderName;

use crate::logging::{noop_header_marker, noop_path_marker, noop_query_marker, HeaderMarker, QueryMarker};

/// A representation of the data marked as sensitive.
#[derive(Clone)]
pub struct Sensitivity<RequestHeader, Path, Query, ResponseHeader> {
    // Request sensitivity markers
    pub(crate) req_header: RequestHeader,
    pub(crate) path: Path,
    pub(crate) query: Query,

    // Response sensitivity markers
    pub(crate) status_code: bool,
    pub(crate) resp_header: ResponseHeader,
}

impl<RequestHeader, Path, QueryKey, ResponseHeader> fmt::Debug
    for Sensitivity<RequestHeader, Path, QueryKey, ResponseHeader>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sensitivity")
            .field("status_code", &self.status_code)
            .finish_non_exhaustive()
    }
}

impl<RequestHeader, Path, Query, ResponseHeader> Sensitivity<RequestHeader, Path, Query, ResponseHeader> {
    /// Marks request headers as sensitive using a closure.
    ///
    /// See [`SensitiveHeaders::mark`](crate::SensitiveHeaders::mark) for more info.
    pub fn request_header<F>(self, marker: F) -> Sensitivity<F, Path, Query, ResponseHeader>
    where
        F: Fn(&HeaderName) -> HeaderMarker,
    {
        Sensitivity {
            req_header: marker,
            path: self.path,
            query: self.query,
            status_code: self.status_code,
            resp_header: self.resp_header,
        }
    }

    /// Marks path segments as sensitive using a closure.
    ///
    /// See [`SensitiveUri::path`](crate::SensitiveUri::path) for more info.
    pub fn path<F>(self, marker: F) -> Sensitivity<RequestHeader, F, Query, ResponseHeader>
    where
        F: Fn(usize) -> bool,
    {
        Sensitivity {
            req_header: self.req_header,
            path: marker,
            query: self.query,
            status_code: self.status_code,
            resp_header: self.resp_header,
        }
    }

    /// Marks parts of the request query parameters as sensitive using a closure.
    ///
    /// See [`SensitiveUri::query`](crate::SensitiveUri::query) for more info.
    pub fn query<F>(self, marker: F) -> Sensitivity<RequestHeader, Path, F, ResponseHeader>
    where
        F: Fn(&str) -> QueryMarker,
    {
        Sensitivity {
            req_header: self.req_header,
            path: self.path,
            query: marker,
            status_code: self.status_code,
            resp_header: self.resp_header,
        }
    }

    /// Marks the response status code as sensitive.
    pub fn status_code(self) -> Sensitivity<RequestHeader, Path, Query, ResponseHeader> {
        Sensitivity {
            req_header: self.req_header,
            path: self.path,
            query: self.query,
            status_code: true,
            resp_header: self.resp_header,
        }
    }

    /// Marks response headers as sensitive using a closure.
    ///
    /// See [`SensitiveHeaders::mark`](crate::SensitiveHeaders::mark) for more info.
    pub fn response_header<F>(self, marker: F) -> Sensitivity<RequestHeader, Path, Query, F>
    where
        F: Fn(&HeaderName) -> HeaderMarker,
    {
        Sensitivity {
            req_header: self.req_header,
            path: self.path,
            query: self.query,
            status_code: self.status_code,
            resp_header: marker,
        }
    }
}

pub(crate) type DefaultSensitivity = Sensitivity<
    fn(&HeaderName) -> HeaderMarker,
    fn(usize) -> bool,
    fn(&str) -> QueryMarker,
    fn(&HeaderName) -> HeaderMarker,
>;

impl Default for DefaultSensitivity {
    fn default() -> Self {
        Self {
            req_header: noop_header_marker,
            path: noop_path_marker,
            query: noop_query_marker,

            status_code: false,
            resp_header: noop_header_marker,
        }
    }
}

impl
    Sensitivity<
        fn(&HeaderName) -> HeaderMarker,
        fn(usize) -> bool,
        fn(&str) -> QueryMarker,
        fn(&HeaderName) -> HeaderMarker,
    >
{
    /// Constructs a new [`Sensitivity`] with nothing marked as sensitive.
    pub fn new() -> Self {
        Self::default()
    }
}
