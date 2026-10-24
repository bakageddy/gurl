/*
 * Hurl (https://hurl.dev)
 * Copyright (C) 2026 Orange
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *          http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
//! `gurl cli`: HURL's own command line, forked from `hurl`'s binary sources (see UPSTREAM.md in
//! this directory) so that it behaves exactly like `hurl`, plus the gURL workspace extras.

mod entry;
mod error;
mod logger;
pub(crate) mod options;
mod run;
mod summary;
mod workspace;

pub use self::entry::{HurlRun, main};
pub(crate) use self::error::CliError;
pub(crate) use self::logger::BaseLogger;
pub(crate) use self::options::OutputType;
pub(crate) use self::summary::summary;
