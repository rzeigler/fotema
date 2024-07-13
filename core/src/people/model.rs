// SPDX-FileCopyrightText: © 2024 David Bliss
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt::Display;
use std::path::PathBuf;

/// Database ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaceId(i64);

impl FaceId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }

    /// FIXME replace this with a To/From SQL implementation.
    pub fn id(&self) -> i64 {
        self.0
    }
}

impl Display for FaceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Person {
    pub person_id: PersonId,
    pub name: String,
}

/// Database ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PersonId(i64);

impl PersonId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }

    /// FIXME replace this with a To/From SQL implementation.
    pub fn id(&self) -> i64 {
        self.0
    }
}

impl Display for PersonId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct Face {
    pub face_id: FaceId,

    /// Path to thumbnail generated from face bounds.
    /// Normalized to be square and expanded to capture the whole head.
    pub thumbnail_path: PathBuf,
    /*
        /// Image cropped from bounds returned by face detection algorithm
        pub bounds_path: PathBuf,

        /// Bounds of detected face.
        pub bounds: Rect,

        /// Confidence (0.0 to 1.0) that the detected face is actually a face.
        pub confidence: f32,

        right_eye: Option<(u32, u32)>,
        left_eye: Option<(u32, u32)>,
        nose: Option<(u32, u32)>,
        right_mouth_corner: Option<(u32, u32)>,
        left_mouth_corner: Option<(u32, u32)>,
    */
}
