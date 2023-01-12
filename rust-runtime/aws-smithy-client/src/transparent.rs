/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![allow(dead_code, missing_docs, missing_debug_implementations)]

use std::marker::PhantomData;
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct TransparentService<S>(S);

impl<S> TransparentService<S> {
    pub fn new(service: S) -> Self {
        Self(service)
    }
}

impl<S, Req> Service<Req> for TransparentService<S>
where
    S: Service<Req>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        self.0.call(req)
    }
}

pub struct TransparentLayer<S>(PhantomData<S>);

impl<S> TransparentLayer<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<S> Layer<S> for TransparentLayer<S> {
    type Service = TransparentService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TransparentService::new(inner)
    }
}
