// Copyright 2017 Peter Williams and collaborators
// Licensed under the MIT License.

/*!
A framework for notifying users about what the framework is doing.

This module provides a way for Blobman to notify the user about actions
taken, problems, and so on. It is very narrowly targeted at the
command-line use case.

This module is ripped off from the `status` module used by the
[Tectonic](https://github.com/tectonic-typesetting/tectonic) typesetting
engine. (Which the author of this module also wrote.)

*/

#[macro_use] pub mod termcolor;

use std::cmp;
use std::fmt::Arguments;

use errors::Error;


/// How chatty the notification system should be.
#[repr(usize)]
#[derive(Clone, Copy, Eq, Debug)]
pub enum ChatterLevel {
    /// A minimal level of output — only warnings and errors will be reported.
    Minimal = 0,

    /// The normal level of output — informational messages will be reported.
    Normal,
}

impl PartialEq for ChatterLevel {
    #[inline]
    fn eq(&self, other: &ChatterLevel) -> bool {
        *self as usize == *other as usize
    }
}

impl PartialOrd for ChatterLevel {
    #[inline]
    fn partial_cmp(&self, other: &ChatterLevel) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ChatterLevel {
    #[inline]
    fn cmp(&self, other: &ChatterLevel) -> cmp::Ordering {
        (*self as usize).cmp(&(*other as usize))
    }
}


/// The kind of notification that is being produced.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NotificationKind {
    /// An informational notice.
    Note,

    /// Warning of an unusual condition; the program will likely perform as intended.
    Warning,

    /// Notification of a severe problem; the program will likely fail but will attempt to contine.
    Severe,

    /// Notification of a fatal error; the program must give up.
    Fatal,
}


/// Trait for type that handle notifications to the user.
pub trait NotificationBackend {
    /// Notify the user about an event.
    ///
    /// If `err` is not `None`, the information contained in the object should
    /// be reported after the main message.
    fn notify(&mut self, kind: NotificationKind, args: Arguments, err: Option<&Error>);
}


/// Send an informational notification to the user.
///
/// Standard usage looks like this:
///
/// ```rust
/// bm_note!(nb, "downloaded {} files", n_files);
/// ```
///
/// where `nb` is a type implementing the NotificationBackend trait. You may
/// also provide an Error value after a semicolon; the information it contains
/// will be printed after the informational message. This is not expected to
/// be common usage for this particular macro, but makes more sense for the
/// `bm_warning!`, `bm_severe!`, and `bm_fatal!` macros.
#[macro_export]
macro_rules! bm_note {
    ($dest:expr, $( $fmt_args:expr ),*) => {
        $dest.notify($crate::status::NotificationKind::Note, format_args!($( $fmt_args ),*), None)
    };
    ($dest:expr, $( $fmt_args:expr ),* ; $err:expr) => {
        $dest.notify($crate::status::NotificationKind::Note, format_args!($( $fmt_args ),*), Some(&$err))
    };
}

/// Warn the user of a problematic condition.
///
/// See the documentation of `bm_note!` for usage information. This macro
/// should be used when an unusual condition has been detected, but the task
/// at hand will likely succeed.
#[macro_export]
macro_rules! bm_warning {
    ($dest:expr, $( $fmt_args:expr ),*) => {
        $dest.notify($crate::status::NotificationKind::Warning, format_args!($( $fmt_args ),*), None)
    };
    ($dest:expr, $( $fmt_args:expr ),* ; $err:expr) => {
        $dest.notify($crate::status::NotificationKind::Warning, format_args!($( $fmt_args ),*), Some(&$err))
    };
}

/// Notify the user of a severe problem.
///
/// See the documentation of `bm_note!` for usage information. This macro
/// should be used when an issue has been detected that makes it likely that
/// the task at hand cannot be completed successfully; however, the program
/// will attempt to continue.
#[macro_export]
macro_rules! bm_severe {
    ($dest:expr, $( $fmt_args:expr ),*) => {
        $dest.notify($crate::status::NotificationKind::Severe, format_args!($( $fmt_args ),*), None)
    };
    ($dest:expr, $( $fmt_args:expr ),* ; $err:expr) => {
        $dest.notify($crate::status::NotificationKind::Severe, format_args!($( $fmt_args ),*), Some(&$err))
    };
}

/// Notify the user of a fatal problem.
///
/// See the documentation of `bm_note!` for usage information. This macro
/// should be used when an issue has been detected that forces the program to
/// give up on the task at hand. If the command-line interface is being used,
/// it will probably exit almost immediately after a fatal notification is
/// issued.
#[macro_export]
macro_rules! bm_fatal {
    ($dest:expr, $( $fmt_args:expr ),*) => {
        $dest.notify($crate::status::NotificationKind::Fatal, format_args!($( $fmt_args ),*), None)
    };
    ($dest:expr, $( $fmt_args:expr ),* ; $err:expr) => {
        $dest.notify($crate::status::NotificationKind::Fatal, format_args!($( $fmt_args ),*), Some(&$err))
    };
}


/// A no-op notification backend.
///
/// This empty structure implements the NotificationBackend trait. Its
/// `notify()` function does nothing.
#[derive(Clone, Copy, Debug)]
pub struct NoopNotificationBackend { }

impl NoopNotificationBackend {
    /// Create a new NoopNotificationBackend object.
    pub fn new() -> NoopNotificationBackend {
        NoopNotificationBackend { }
    }
}

impl NotificationBackend for NoopNotificationBackend {
    fn notify(&mut self, _kind: NotificationKind, _args: Arguments, _err: Option<&Error>) {}
}
