//! Scene module for 3D rendering.

use crate::audio::{SharedSongPlayer, SongPlayer};
use crate::camera::{Camera3D, FretCamera};
use crate::midi::MidiHandler;
use crate::song::{
    DrumArticulationSimple, DrumKitPieceSimple, SongBeat, SongChord, SongDrumNote,
    SongInstrumentType, SongKeyboardNote, SongNote, SongNoteTechnique,
};
use crate::{
    DEFAULT_NOTE_DISPLAY_DISTANCE, DEFAULT_NOTE_DISPLAY_SECONDS, DRUM_NOTE_DISPLAY_DISTANCE,
    DRUM_NOTE_DISPLAY_SECONDS,
};
use bytemuck::{Pod, Zeroable};
use cgmath::{InnerSpace, Matrix4, Vector3, Vector4};
use std::collections::{HashMap, HashSet};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct SceneVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub tex_coords: [f32; 2],
}

#[derive(Debug, Clone, Copy)]
pub struct SpriteRegion {
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct SpriteFontGlyph {
    pub region: SpriteRegion,
}

#[derive(Debug, Clone)]
pub struct SpriteFontDefinition {
    pub line_height: f32,
    pub spacing: f32,
    glyphs: HashMap<char, SpriteFontGlyph>,
}

impl SpriteFontDefinition {
    pub fn new(line_height: f32, spacing: f32) -> Self {
        Self {
            line_height,
            spacing,
            glyphs: HashMap::new(),
        }
    }

    pub fn insert_glyph(&mut self, character: char, glyph: SpriteFontGlyph) {
        self.glyphs.insert(character, glyph);
    }

    pub fn glyph(&self, character: char) -> Option<SpriteFontGlyph> {
        self.glyphs
            .get(&character)
            .copied()
            .or_else(|| self.glyphs.get(&'?').copied())
            .or_else(|| self.glyphs.get(&' ').copied())
    }

    pub fn measure_text(&self, text: &str, image_scale: f32) -> (f32, f32) {
        let mut width = 0.0;
        let mut visible_glyphs = 0;

        for character in text.chars() {
            if let Some(glyph) = self.glyph(character) {
                width += glyph.region.width as f32;
                visible_glyphs += 1;
            }
        }

        if visible_glyphs > 1 {
            width += self.spacing * (visible_glyphs - 1) as f32;
        }

        (width * image_scale, self.line_height * image_scale)
    }
}

#[derive(Debug, Clone)]
pub struct SpriteLibrary {
    sprites: HashMap<String, SpriteRegion>,
    fonts: HashMap<String, SpriteFontDefinition>,
}

impl SpriteLibrary {
    pub fn new() -> Self {
        Self {
            sprites: HashMap::new(),
            fonts: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, region: SpriteRegion) {
        self.sprites.insert(name, region);
    }

    pub fn insert_font(&mut self, name: String, font: SpriteFontDefinition) {
        self.fonts.insert(name, font);
    }

    pub fn sprite(&self, name: &str) -> SpriteRegion {
        self.sprites
            .get(name)
            .copied()
            .unwrap_or_else(|| self.white_pixel())
    }

    pub fn white_pixel(&self) -> SpriteRegion {
        self.sprites
            .get("SingleWhitePixel")
            .copied()
            .expect("SingleWhitePixel sprite missing from atlas")
    }

    pub fn font(&self, name: &str) -> Option<&SpriteFontDefinition> {
        self.fonts.get(name)
    }
}

impl Default for SpriteLibrary {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Scene3D {
    pub camera: Camera3D,
    pub fog_enabled: bool,
    pub fog_start: f32,
    pub fog_end: f32,
    pub fog_color: [f32; 4],
}

impl Scene3D {
    pub fn new() -> Self {
        let mut camera = Camera3D::new();
        camera.position = Vector3::new(0.0, 0.0, 5.0);
        camera.forward = Vector3::new(0.0, 0.0, -1.0);

        Self {
            camera,
            fog_enabled: false,
            fog_start: 400.0,
            fog_end: 700.0,
            fog_color: [0.0, 0.0, 0.0, 1.0],
        }
    }

    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        self.camera.get_projection_matrix()
    }

    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        self.camera.get_view_matrix()
    }

    fn project_point(&self, point: Vector3<f32>) -> Option<[f32; 2]> {
        let clip = self.get_projection_matrix()
            * self.get_view_matrix()
            * Vector4::new(point.x, point.y, point.z, 1.0);
        if clip.w <= 0.01 {
            return None;
        }

        Some([clip.x / clip.w, clip.y / clip.w])
    }

    fn push_world_quad(
        &self,
        vertices: &mut Vec<SceneVertex>,
        bottom_left: Vector3<f32>,
        top_left: Vector3<f32>,
        top_right: Vector3<f32>,
        bottom_right: Vector3<f32>,
        color: [f32; 4],
        sprite: SpriteRegion,
    ) {
        let Some(bottom_left) = self.project_point(bottom_left) else {
            return;
        };
        let Some(top_left) = self.project_point(top_left) else {
            return;
        };
        let Some(top_right) = self.project_point(top_right) else {
            return;
        };
        let Some(bottom_right) = self.project_point(bottom_right) else {
            return;
        };

        vertices.extend_from_slice(&[
            SceneVertex {
                position: bottom_left,
                color,
                tex_coords: [sprite.u0, sprite.v1],
            },
            SceneVertex {
                position: top_left,
                color,
                tex_coords: [sprite.u0, sprite.v0],
            },
            SceneVertex {
                position: top_right,
                color,
                tex_coords: [sprite.u1, sprite.v0],
            },
            SceneVertex {
                position: bottom_left,
                color,
                tex_coords: [sprite.u0, sprite.v1],
            },
            SceneVertex {
                position: top_right,
                color,
                tex_coords: [sprite.u1, sprite.v0],
            },
            SceneVertex {
                position: bottom_right,
                color,
                tex_coords: [sprite.u1, sprite.v1],
            },
        ]);
    }

    fn push_world_nine_patch(
        &self,
        vertices: &mut Vec<SceneVertex>,
        bottom_left: Vector3<f32>,
        top_left: Vector3<f32>,
        top_right: Vector3<f32>,
        _bottom_right: Vector3<f32>,
        color: [f32; 4],
        sprite: SpriteRegion,
    ) {
        let x_tex_coords = [
            sprite.u0,
            (sprite.u0 + sprite.u1) * 0.5,
            (sprite.u0 + sprite.u1) * 0.5,
            sprite.u1,
        ];
        let y_tex_coords = [
            sprite.v0,
            (sprite.v0 + sprite.v1) * 0.5,
            (sprite.v0 + sprite.v1) * 0.5,
            sprite.v1,
        ];
        let patch_percents = [0.0, 0.05, 0.95, 1.0];
        let top_left_to_top_right = top_right - top_left;
        let top_left_to_bottom_left = bottom_left - top_left;

        for x in 0..3 {
            for y in 0..3 {
                let patch_top_left = top_left
                    + top_left_to_top_right * patch_percents[x]
                    + top_left_to_bottom_left * patch_percents[y];
                let patch_top_right = top_left
                    + top_left_to_top_right * patch_percents[x + 1]
                    + top_left_to_bottom_left * patch_percents[y];
                let patch_bottom_left = top_left
                    + top_left_to_top_right * patch_percents[x]
                    + top_left_to_bottom_left * patch_percents[y + 1];
                let patch_bottom_right = top_left
                    + top_left_to_top_right * patch_percents[x + 1]
                    + top_left_to_bottom_left * patch_percents[y + 1];

                self.push_world_quad(
                    vertices,
                    patch_bottom_left,
                    patch_top_left,
                    patch_top_right,
                    patch_bottom_right,
                    color,
                    SpriteRegion {
                        u0: x_tex_coords[x],
                        v0: y_tex_coords[y],
                        u1: x_tex_coords[x + 1],
                        v1: y_tex_coords[y + 1],
                        ..sprite
                    },
                );
            }
        }
    }

    fn push_billboard(
        &self,
        vertices: &mut Vec<SceneVertex>,
        center: Vector3<f32>,
        image_scale: f32,
        color: [f32; 4],
        sprite: SpriteRegion,
    ) {
        let right = if self.camera.right.magnitude2() > 0.0 {
            self.camera.right.normalize()
        } else {
            Vector3::unit_x()
        };
        let up = if self.camera.up.magnitude2() > 0.0 {
            self.camera.up.normalize()
        } else {
            Vector3::unit_y()
        };

        let half_width = sprite.width as f32 * image_scale;
        let half_height = sprite.height as f32 * image_scale;
        self.push_world_quad(
            vertices,
            center - right * half_width - up * half_height,
            center - right * half_width + up * half_height,
            center + right * half_width + up * half_height,
            center + right * half_width - up * half_height,
            color,
            sprite,
        );
    }
}

impl Default for Scene3D {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ChartScene3D {
    pub base: Scene3D,
    pub note_display_seconds: f32,
    pub note_display_distance: f32,
    pub num_notes_detected: i32,
    pub num_notes_total: i32,
    pub current_bpm: f32,
    pub lefty_mode: bool,
    pub current_time_offset: f32,
    pub player: Option<SharedSongPlayer>,
    pub time_scale: f32,
    pub current_time: f32,
    pub start_time: f32,
    pub end_time: f32,
    pub highway_start_x: f32,
    pub highway_end_x: f32,
    pub score_start_secs: f32,
    pub start_beat_position: i32,
    pub white_half_alpha: [f32; 4],
    pub white_three_quarters_alpha: [f32; 4],
}

impl ChartScene3D {
    pub fn new(player: Option<SharedSongPlayer>) -> Self {
        Self {
            base: Scene3D::new(),
            note_display_seconds: DEFAULT_NOTE_DISPLAY_SECONDS,
            note_display_distance: DEFAULT_NOTE_DISPLAY_DISTANCE,
            num_notes_detected: 0,
            num_notes_total: 0,
            current_bpm: 0.0,
            lefty_mode: false,
            current_time_offset: 0.0,
            player,
            time_scale: 0.0,
            current_time: 0.0,
            start_time: 0.0,
            end_time: 0.0,
            highway_start_x: 0.0,
            highway_end_x: 0.0,
            score_start_secs: 0.0,
            start_beat_position: 0,
            white_half_alpha: [1.0, 1.0, 1.0, 0.5],
            white_three_quarters_alpha: [1.0, 1.0, 1.0, 0.75],
        }
    }

    pub fn reset_score(&mut self, score_start_secs: f32) {
        self.score_start_secs = score_start_secs;
        self.num_notes_detected = 0;
        self.num_notes_total = 0;
    }

    pub fn begin_frame(&mut self, current_time: f32, viewport_width: u32, viewport_height: u32) {
        self.time_scale = self.note_display_distance / self.note_display_seconds;
        self.current_time = current_time;
        self.start_time = current_time;
        self.end_time = current_time + self.note_display_seconds;
        self.base.camera.viewport_width = viewport_width.max(1) as i32;
        self.base.camera.viewport_height = viewport_height.max(1) as i32;
        self.base.camera.mirror_left_right = self.lefty_mode;
    }

    pub fn current_playback_seconds(&self) -> f32 {
        self.player
            .as_ref()
            .and_then(|player| {
                player
                    .lock()
                    .ok()
                    .map(|player| player.current_output_second())
            })
            .unwrap_or(self.current_time)
    }

    pub fn player_handle(&self) -> Option<SharedSongPlayer> {
        self.player.as_ref().cloned()
    }

    pub fn get_start_note<T: SongEventTrait>(
        &self,
        time_offset: f32,
        min_length: f32,
        start_note_position: i32,
        notes: &[T],
    ) -> i32 {
        if notes.is_empty() {
            return 0;
        }

        let mut position = start_note_position.clamp(0, notes.len() as i32 - 1) as usize;
        while position > 0 {
            let end_time = notes[position]
                .end_time()
                .max(notes[position].time_offset() + min_length);
            if end_time < time_offset {
                break;
            }
            position -= 1;
        }

        while position < notes.len() {
            let end_time = notes[position]
                .end_time()
                .max(notes[position].time_offset() + min_length);
            if end_time > time_offset {
                break;
            }
            position += 1;
        }

        position as i32
    }

    pub fn get_end_note<T: SongEventTrait>(
        &self,
        start_position: i32,
        end_time: f32,
        notes: &[T],
    ) -> i32 {
        if notes.is_empty() {
            return 0;
        }

        let mut position = start_position.max(0) as usize;
        while position < notes.len() {
            if notes[position].time_offset() > end_time {
                break;
            }
            position += 1;
        }

        if position == notes.len() {
            position -= 1;
        }

        position as i32
    }

    fn push_beat_lines(
        &mut self,
        vertices: &mut Vec<SceneVertex>,
        sprites: &SpriteLibrary,
        height_offset: f32,
        image_scale: f32,
    ) {
        let Some(player) = self.player.as_ref() else {
            return;
        };

        let beats = player
            .lock()
            .ok()
            .map(|player| player.get_song_beats())
            .unwrap_or_default();
        if beats.is_empty() {
            return;
        }

        self.start_beat_position = self.get_start_note(
            self.current_time - self.current_time_offset,
            0.0,
            self.start_beat_position,
            &beats,
        );
        let line_sprite = sprites.sprite("HorizontalFretLine");

        let mut last_beat_time = None;
        self.current_bpm = 0.0;
        for beat in beats.iter().skip(self.start_beat_position.max(0) as usize) {
            if beat.time_offset > self.end_time {
                break;
            }

            let mut color = [1.0, 1.0, 1.0, 0.25];
            if beat.is_measure {
                color[3] = 0.5;
            }
            self.push_horizontal_line(
                vertices,
                self.highway_start_x,
                self.highway_end_x,
                beat.time_offset,
                height_offset,
                color,
                line_sprite,
                image_scale,
            );

            if let Some(last) = last_beat_time {
                if self.current_bpm == 0.0 {
                    let delta = beat.time_offset - last;
                    if delta > 0.0 {
                        self.current_bpm = (1.0 / delta) * 60.0;
                    }
                }
            }
            last_beat_time = Some(beat.time_offset);
        }
    }

    fn push_horizontal_line(
        &self,
        vertices: &mut Vec<SceneVertex>,
        start_x: f32,
        end_x: f32,
        time: f32,
        height_offset: f32,
        color: [f32; 4],
        sprite: SpriteRegion,
        image_scale: f32,
    ) {
        let z = -(time * self.time_scale);
        let half_depth = sprite.height as f32 * image_scale;
        self.base.push_world_quad(
            vertices,
            Vector3::new(start_x, height_offset, z + half_depth),
            Vector3::new(start_x, height_offset, z - half_depth),
            Vector3::new(end_x, height_offset, z - half_depth),
            Vector3::new(end_x, height_offset, z + half_depth),
            color,
            sprite,
        );
    }
}

impl Default for ChartScene3D {
    fn default() -> Self {
        Self::new(None)
    }
}

pub trait SongEventTrait {
    fn time_offset(&self) -> f32;
    fn end_time(&self) -> f32;
}

impl SongEventTrait for SongBeat {
    fn time_offset(&self) -> f32 {
        self.time_offset
    }
    fn end_time(&self) -> f32 {
        self.time_offset
    }
}

impl SongEventTrait for SongNote {
    fn time_offset(&self) -> f32 {
        self.time_offset
    }
    fn end_time(&self) -> f32 {
        self.end_time
    }
}

impl SongEventTrait for SongDrumNote {
    fn time_offset(&self) -> f32 {
        self.time_offset
    }
    fn end_time(&self) -> f32 {
        self.end_time
    }
}

impl SongEventTrait for SongKeyboardNote {
    fn time_offset(&self) -> f32 {
        self.time_offset
    }
    fn end_time(&self) -> f32 {
        self.end_time
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DrumHit {
    pub channel: i32,
    pub note_number: i32,
    pub velocity: f32,
    pub sample_offset: i32,
    pub is_live: bool,
    pub dimension_value: f32,
}

#[derive(Debug, Clone)]
pub struct DrumPlayerScene3D {
    pub base: ChartScene3D,
    num_lanes: i32,
    camera_distance: f32,
    position_lane: f32,
    notes_detected: Vec<Option<f32>>,
    start_note_position: i32,
    detection_tolerance_secs: f32,
    drum_hits: Vec<DrumHit>,
}

impl DrumPlayerScene3D {
    pub fn new(player: SharedSongPlayer) -> Self {
        let num_notes = player
            .lock()
            .ok()
            .map(|player| player.get_drum_notes().len())
            .unwrap_or(0);
        let mut base = ChartScene3D::new(Some(player));
        base.current_time_offset = 0.35;
        base.note_display_distance = DRUM_NOTE_DISPLAY_DISTANCE;
        base.note_display_seconds = DRUM_NOTE_DISPLAY_SECONDS;
        base.highway_start_x = 0.0;
        base.highway_end_x = 75.0;

        Self {
            base,
            num_lanes: 5,
            camera_distance: 75.0,
            position_lane: 2.5,
            notes_detected: vec![None; num_notes],
            start_note_position: 0,
            detection_tolerance_secs: 0.1,
            drum_hits: Vec::new(),
        }
    }

    pub fn handle_drum_hit(&mut self, hit: DrumHit) {
        self.drum_hits.push(hit);
    }

    pub fn get_lane_position(&self, lane: f32) -> f32 {
        lane * 15.0
    }

    pub fn build_vertices(
        &mut self,
        current_time: f32,
        viewport_width: u32,
        viewport_height: u32,
        sprites: &SpriteLibrary,
    ) -> Vec<SceneVertex> {
        self.base
            .begin_frame(current_time, viewport_width, viewport_height);
        self.base.highway_start_x = 0.0;
        self.base.highway_end_x = self.get_lane_position(self.num_lanes as f32);
        self.update_camera();

        let mut vertices = Vec::with_capacity(1024);
        let background = sprites.white_pixel();
        self.base.base.push_world_quad(
            &mut vertices,
            Vector3::new(-10.0, -20.0, 400.0),
            Vector3::new(-10.0, 120.0, 400.0),
            Vector3::new(110.0, 120.0, 400.0),
            Vector3::new(110.0, -20.0, 400.0),
            [0.05, 0.06, 0.10, 1.0],
            background,
        );

        let vertical_line = sprites.sprite("VerticalFretLine");
        for lane in 0..=self.num_lanes {
            self.push_lane_timeline(
                &mut vertices,
                lane as f32,
                self.base.current_time - self.base.current_time_offset,
                self.base.end_time,
                [1.0, 1.0, 1.0, 0.35],
                vertical_line,
                0.03,
            );
        }

        self.base.push_beat_lines(&mut vertices, sprites, 0.0, 0.08);

        if let Some(player) = self.base.player.as_ref() {
            let notes = player
                .lock()
                .ok()
                .map(|player| player.get_drum_notes())
                .unwrap_or_default();
            if !notes.is_empty() {
                self.start_note_position = self.base.get_start_note(
                    self.base.current_time - 0.5,
                    0.0,
                    self.start_note_position,
                    &notes,
                );
                let last_note =
                    self.base
                        .get_end_note(self.start_note_position, self.base.end_time, &notes);

                for (idx, note) in notes
                    .iter()
                    .enumerate()
                    .take(last_note.max(0) as usize + 1)
                    .skip(self.start_note_position.max(0) as usize)
                {
                    if note.time_offset <= self.base.current_time - self.detection_tolerance_secs
                        && self.notes_detected[idx].is_none()
                        && note.time_offset > self.base.score_start_secs
                    {
                        self.notes_detected[idx] = Some(f32::MAX);
                        self.base.num_notes_total += 1;
                    }

                    if note.time_offset > self.base.end_time {
                        break;
                    }

                    let scale = if note.time_offset <= self.base.current_time {
                        let delta = self.base.current_time - note.time_offset;
                        if delta < 0.1 {
                            1.0 + ((1.0 - (delta / 0.1)) * 1.5)
                        } else {
                            1.0
                        }
                    } else {
                        1.0
                    };

                    self.push_drum_note(&mut vertices, note, scale, sprites);
                }
            }
        }

        let horizontal = sprites.sprite("HorizontalFretLine");
        self.base.push_horizontal_line(
            &mut vertices,
            0.0,
            self.get_lane_position(self.num_lanes as f32),
            self.base.start_time,
            0.0,
            [1.0, 1.0, 1.0, 0.8],
            horizontal,
            0.04,
        );
        vertices
    }

    fn update_camera(&mut self) {
        let target_position = self.num_lanes as f32 / 2.0;
        self.position_lane = lerp(self.position_lane, target_position, 0.01);
        let front_position =
            -((self.base.current_time - self.base.current_time_offset) * self.base.time_scale);
        self.base.base.camera.position = Vector3::new(
            self.get_lane_position(self.position_lane),
            150.0,
            front_position + self.camera_distance,
        );
        self.base.base.camera.set_look_at(Vector3::new(
            self.get_lane_position(self.position_lane),
            0.0,
            self.base.base.camera.position.z - (self.base.note_display_distance * 0.55),
        ));
    }

    fn push_lane_timeline(
        &self,
        vertices: &mut Vec<SceneVertex>,
        lane: f32,
        start_time: f32,
        end_time: f32,
        color: [f32; 4],
        sprite: SpriteRegion,
        image_scale: f32,
    ) {
        let x = self.get_lane_position(lane);
        let start_z = -(start_time * self.base.time_scale);
        let end_z = -(end_time * self.base.time_scale);
        let half_width = sprite.width as f32 * image_scale;
        self.base.base.push_world_quad(
            vertices,
            Vector3::new(x - half_width, 0.0, start_z),
            Vector3::new(x - half_width, 0.0, end_z),
            Vector3::new(x + half_width, 0.0, end_z),
            Vector3::new(x + half_width, 0.0, start_z),
            color,
            sprite,
        );
    }

    fn push_drum_note(
        &self,
        vertices: &mut Vec<SceneVertex>,
        note: &SongDrumNote,
        scale: f32,
        sprites: &SpriteLibrary,
    ) {
        if note.kit_piece == DrumKitPieceSimple::Kick {
            let sprite = sprites.sprite("HorizontalFretLine");
            self.base.push_horizontal_line(
                vertices,
                self.get_lane_position(0.25),
                self.get_lane_position(self.num_lanes as f32 - 0.25),
                note.time_offset,
                0.0,
                [0.95, 0.86, 0.18, 1.0],
                sprite,
                0.08 * scale * scale,
            );
            return;
        }

        let (lane, sprite_name) = match note.kit_piece {
            DrumKitPieceSimple::Snare => (0.5, "DrumRed"),
            DrumKitPieceSimple::HiHat => (
                1.5,
                match note.articulation {
                    DrumArticulationSimple::HiHatOpen => "CymbalYellowOpen",
                    _ => "CymbalYellow",
                },
            ),
            DrumKitPieceSimple::Crash => (2.5, "CymbalGreen"),
            DrumKitPieceSimple::Ride => (
                3.5,
                match note.articulation {
                    DrumArticulationSimple::CymbalBell => "CymbalBlueBell",
                    _ => "CymbalBlue",
                },
            ),
            DrumKitPieceSimple::Tom1 => (1.5, "DrumYellow"),
            DrumKitPieceSimple::Tom2 => (2.5, "DrumGreen"),
            DrumKitPieceSimple::Tom3 => (3.5, "DrumBlue"),
            DrumKitPieceSimple::Kick => unreachable!(),
        };

        let sprite = sprites.sprite(sprite_name);
        let center = Vector3::new(
            self.get_lane_position(lane),
            0.0,
            -(note.time_offset * self.base.time_scale),
        );
        self.base
            .base
            .push_billboard(vertices, center, 0.08 * scale, [1.0, 1.0, 1.0, 1.0], sprite);
    }
}

impl MidiHandler for DrumPlayerScene3D {
    fn handle_note_on(
        &mut self,
        channel: i32,
        note_number: i32,
        velocity: f32,
        sample_offset: i32,
    ) {
        self.handle_drum_hit(DrumHit {
            channel,
            note_number,
            velocity,
            sample_offset,
            is_live: true,
            dimension_value: 0.0,
        });
    }

    fn handle_poly_pressure(
        &mut self,
        channel: i32,
        note_number: i32,
        pressure: f32,
        sample_offset: i32,
    ) {
        self.handle_drum_hit(DrumHit {
            channel,
            note_number,
            velocity: pressure,
            sample_offset,
            is_live: true,
            dimension_value: 0.0,
        });
    }
}

impl Default for DrumPlayerScene3D {
    fn default() -> Self {
        Self::new(SongPlayer::shared_demo_for(SongInstrumentType::Drums))
    }
}

#[derive(Debug, Clone)]
pub struct FretPlayerScene3D {
    pub base: ChartScene3D,
    pub display_notes: bool,
    pub detect_semitone_offset: f32,
    pub capo_fret: i32,
    num_frets: i32,
    num_strings: i32,
    min_fret: f32,
    max_fret: f32,
    start_note_position: i32,
    string_colors: [[f32; 4]; 6],
    string_color_names: Vec<String>,
    string_color_offset: i32,
    target_focus_fret: f32,
    fret_camera: FretCamera,
}

impl FretPlayerScene3D {
    pub fn new(player: SharedSongPlayer) -> Self {
        let (_, num_strings) = player
            .lock()
            .ok()
            .map(|player| {
                (
                    player.get_instrument_notes().len(),
                    if player.instrument_type == SongInstrumentType::BassGuitar {
                        4
                    } else {
                        6
                    },
                )
            })
            .unwrap_or((0, 6));
        let mut base = ChartScene3D::new(Some(player));
        base.highway_start_x = fret_position(0.0);
        base.highway_end_x = fret_position(24.0);

        Self {
            base,
            display_notes: true,
            detect_semitone_offset: 0.0,
            capo_fret: 0,
            num_frets: 24,
            num_strings,
            min_fret: 0.0,
            max_fret: 4.0,
            start_note_position: 0,
            string_colors: [
                [0.10, 0.80, 0.00, 1.0],
                [1.00, 0.00, 0.00, 1.0],
                [1.00, 1.00, 0.00, 1.0],
                [0.00, 0.60, 1.00, 1.0],
                [1.00, 0.50, 0.00, 1.0],
                [0.80, 0.00, 0.80, 1.0],
            ],
            string_color_names: vec![
                "Green".into(),
                "Red".into(),
                "Yellow".into(),
                "Cyan".into(),
                "Orange".into(),
                "Purple".into(),
            ],
            string_color_offset: 0,
            target_focus_fret: 2.0,
            fret_camera: FretCamera::new(),
        }
    }

    pub fn get_string_height(&self, string: f32) -> f32 {
        3.0 + (string * 4.0)
    }

    pub fn get_fret_position(&self, fret: f32) -> f32 {
        fret_position(fret)
    }

    pub fn build_vertices(
        &mut self,
        current_time: f32,
        viewport_width: u32,
        viewport_height: u32,
        sprites: &SpriteLibrary,
    ) -> Vec<SceneVertex> {
        self.base
            .begin_frame(current_time, viewport_width, viewport_height);
        self.base.highway_start_x = self.get_fret_position(0.0);
        self.base.highway_end_x = self.get_fret_position(self.num_frets as f32);
        self.refresh_fret_window();
        self.update_camera();

        let white = sprites.white_pixel();
        let vertical = sprites.sprite("VerticalFretLine");
        let horizontal = sprites.sprite("HorizontalFretLine");

        let mut vertices = Vec::with_capacity(4096);
        self.base.base.push_world_quad(
            &mut vertices,
            Vector3::new(self.base.highway_start_x - 20.0, -10.0, 220.0),
            Vector3::new(self.base.highway_start_x - 20.0, 35.0, 220.0),
            Vector3::new(self.base.highway_end_x + 20.0, 35.0, 220.0),
            Vector3::new(self.base.highway_end_x + 20.0, -10.0, 220.0),
            [0.06, 0.05, 0.04, 1.0],
            white,
        );

        for fret in 0..self.num_frets {
            self.push_fret_timeline(
                &mut vertices,
                fret as f32,
                self.base.start_time,
                self.base.end_time,
                [1.0, 1.0, 1.0, 0.35],
                vertical,
                0.03,
            );
        }

        self.base.push_beat_lines(&mut vertices, sprites, 0.0, 0.08);

        for string in 0..self.num_strings {
            let mut color = self.get_string_color(string as usize);
            color[3] = 0.75;
            self.base.push_horizontal_line(
                &mut vertices,
                self.get_fret_position(0.0),
                self.get_fret_position(self.num_frets as f32),
                self.base.start_time,
                self.get_string_height(string as f32),
                color,
                horizontal,
                0.04,
            );
        }

        for fret in 1..self.num_frets {
            self.push_fret_marker(
                &mut vertices,
                fret as f32 - 1.0,
                self.base.start_time,
                self.get_string_height(0.0),
                self.get_string_height(self.num_strings as f32 - 1.0),
                [1.0, 1.0, 1.0, 0.35],
                vertical,
                0.03,
            );
        }

        if self.capo_fret != 0 {
            self.push_fret_marker(
                &mut vertices,
                self.capo_fret as f32 - 0.1,
                self.base.start_time,
                self.get_string_height(-0.2),
                self.get_string_height(self.num_strings as f32 - 0.8),
                [1.0, 1.0, 1.0, 0.75],
                vertical,
                0.08,
            );
        }

        if self.display_notes {
            self.push_notes(&mut vertices, sprites, vertical);
        }

        vertices
    }

    fn refresh_fret_window(&mut self) {
        self.min_fret = self.num_frets as f32;
        self.max_fret = 0.0;

        let Some(player) = self.base.player.as_ref() else {
            return;
        };
        let notes = player
            .lock()
            .ok()
            .map(|player| player.get_instrument_notes())
            .unwrap_or_default();
        if notes.is_empty() {
            self.min_fret = 0.0;
            self.max_fret = 4.0;
            return;
        }

        self.start_note_position = self.base.get_start_note(
            self.base.current_time,
            1.0,
            self.start_note_position,
            &notes,
        );
        let end_position =
            self.base
                .get_end_note(self.start_note_position, self.base.end_time, &notes);
        let mut first_future_hand_fret = None;

        for note in notes
            .iter()
            .take(end_position.max(0) as usize + 1)
            .skip(self.start_note_position.max(0) as usize)
        {
            if note.time_offset > self.base.end_time {
                break;
            }

            if note.time_offset > self.base.current_time && first_future_hand_fret.is_none() {
                first_future_hand_fret = Some(note.hand_fret as f32 + 1.0);
            }

            if note.fret > 0 {
                self.min_fret = self.min_fret.min(note.fret as f32);
                self.max_fret = self.max_fret.max(note.fret as f32);
            } else {
                self.min_fret = self.min_fret.min(note.hand_fret as f32);
                self.max_fret = self.max_fret.max(note.hand_fret as f32 + 3.0);
            }
        }

        if self.min_fret > self.max_fret {
            self.min_fret = 0.0;
            self.max_fret = 4.0;
        }

        if let Some(target) = first_future_hand_fret {
            self.target_focus_fret = target;
        }
    }

    fn update_camera(&mut self) {
        self.fret_camera.base.viewport_width = self.base.base.camera.viewport_width;
        self.fret_camera.base.viewport_height = self.base.base.camera.viewport_height;
        self.fret_camera.base.mirror_left_right = self.base.lefty_mode;
        self.fret_camera.update(
            self.min_fret,
            self.max_fret,
            self.target_focus_fret,
            -(self.base.current_time * self.base.time_scale),
        );
        self.base.base.camera = self.fret_camera.base.clone();
    }

    fn push_notes(
        &mut self,
        vertices: &mut Vec<SceneVertex>,
        sprites: &SpriteLibrary,
        stem_sprite: SpriteRegion,
    ) {
        let Some(player) = self.base.player.as_ref() else {
            return;
        };
        let notes = player
            .lock()
            .ok()
            .map(|player| player.get_instrument_notes())
            .unwrap_or_default();
        if notes.is_empty() {
            return;
        }

        let chords = player
            .lock()
            .ok()
            .map(|player| player.get_chords())
            .unwrap_or_default();
        let mut drawn_chords = HashSet::new();

        let end_position =
            self.base
                .get_end_note(self.start_note_position, self.base.end_time, &notes);
        for note in notes
            .iter()
            .take(end_position.max(0) as usize + 1)
            .skip(self.start_note_position.max(0) as usize)
        {
            if note.time_offset > self.base.end_time {
                break;
            }

            if note.has_technique(SongNoteTechnique::CHORD) && note.chord_id >= 0 {
                let chord_key = (note.chord_id, note.time_offset.to_bits());
                if drawn_chords.insert(chord_key) {
                    if let Some(chord) = chords.get(note.chord_id as usize) {
                        self.push_chord_outline(vertices, note, chord, sprites);
                        self.push_chord_notes(vertices, note, chord, sprites, stem_sprite);
                        self.push_finger_overlays(vertices, note, chord, sprites);
                    }
                }
                continue;
            }

            self.push_single_note(vertices, note, sprites, stem_sprite, false);

            if note.finger_id >= 0 {
                if let Some(chord) = chords.get(note.finger_id as usize) {
                    self.push_chord_outline(vertices, note, chord, sprites);
                    self.push_finger_overlays(vertices, note, chord, sprites);
                }
            }
        }
    }

    fn push_single_note(
        &self,
        vertices: &mut Vec<SceneVertex>,
        note: &SongNote,
        sprites: &SpriteLibrary,
        stem_sprite: SpriteRegion,
        is_ghost: bool,
    ) {
        let string_offset = self.get_string_offset(note.string).max(0) as f32;
        let string_height = self.get_note_head_height(note, string_offset);
        let note_head_time = note.time_offset.max(self.base.current_time);
        let note_end_time = (note.time_offset + note.time_length).max(note_head_time);
        let draw_fret = self.get_draw_fret(note, note_head_time);
        let string_color = self.get_note_color(note, is_ghost);
        let head_name = if note.time_offset <= self.base.current_time && !is_ghost {
            "GuitarDetected".to_string()
        } else {
            self.note_head_sprite_name(note.string.max(0) as usize)
        };
        let trail_name = self.note_trail_sprite_name(note.string.max(0) as usize);
        let head_sprite = sprites.sprite(head_name.as_str());
        let trail_sprite = sprites.sprite(trail_name.as_str());

        if note_end_time > note_head_time {
            if note.has_technique(SongNoteTechnique::SLIDE) && note.slide_fret >= 0 {
                self.push_slide_tail(
                    vertices,
                    note,
                    draw_fret,
                    note_head_time,
                    note_end_time.min(self.base.end_time),
                    string_height,
                    trail_sprite,
                    string_color,
                );
            } else if note
                .cents_offsets
                .as_ref()
                .is_some_and(|offsets| !offsets.is_empty())
            {
                self.push_bend_trail(
                    vertices,
                    note,
                    string_offset,
                    note_end_time.min(self.base.end_time),
                    trail_sprite,
                    string_color,
                );
            } else if note.has_technique(SongNoteTechnique::VIBRATO) {
                self.push_vibrato_trail(
                    vertices,
                    draw_fret,
                    note_head_time,
                    note_end_time.min(self.base.end_time),
                    string_height,
                    trail_sprite,
                    string_color,
                );
            } else {
                self.push_fret_tail(
                    vertices,
                    note,
                    draw_fret,
                    note_head_time,
                    note_end_time.min(self.base.end_time),
                    string_height,
                    trail_sprite,
                    string_color,
                );
            }
        }

        self.push_fret_head(
            vertices,
            note,
            draw_fret,
            note_head_time,
            string_height,
            head_sprite,
            [1.0, 1.0, 1.0, if is_ghost { 0.4 } else { 1.0 }],
        );

        if let Some(modifier) = self.note_modifier_sprite(note, sprites) {
            self.push_modifier(
                vertices,
                draw_fret,
                note_head_time,
                self.get_string_height(string_offset),
                modifier,
            );
        }

        if note.time_offset > self.base.current_time {
            self.push_note_shadow(vertices, draw_fret, note_head_time, string_offset, sprites);

            if note.fret > 0 {
                let fret_text = note.fret.to_string();
                self.push_vertical_text(
                    vertices,
                    fret_text.as_str(),
                    draw_fret - 0.5,
                    0.0,
                    note_head_time,
                    [1.0, 1.0, 1.0, 1.0],
                    0.12,
                    false,
                    sprites,
                );
            }
        }

        self.push_fret_marker(
            vertices,
            draw_fret - 0.5,
            note_head_time,
            0.0,
            self.get_string_height(string_offset),
            self.base.white_half_alpha,
            stem_sprite,
            0.03,
        );
    }

    fn push_chord_notes(
        &self,
        vertices: &mut Vec<SceneVertex>,
        note: &SongNote,
        chord: &SongChord,
        sprites: &SpriteLibrary,
        stem_sprite: SpriteRegion,
    ) {
        for (string_index, fret) in chord.frets.iter().enumerate() {
            if *fret < 0 || string_index >= self.num_strings as usize {
                continue;
            }

            let chord_note = SongNote {
                chord_id: note.chord_id,
                time_offset: note.time_offset,
                time_length: note.time_length,
                end_time: note.end_time,
                fret: *fret,
                string: string_index as i32,
                techniques: note.techniques | SongNoteTechnique::CHORD_NOTE,
                hand_fret: note.hand_fret,
                finger_id: note.finger_id,
                slide_fret: -1,
                cents_offsets: None,
            };
            self.push_single_note(vertices, &chord_note, sprites, stem_sprite, false);
        }
    }

    fn push_chord_outline(
        &self,
        vertices: &mut Vec<SceneVertex>,
        note: &SongNote,
        chord: &SongChord,
        sprites: &SpriteLibrary,
    ) {
        let sprite = sprites.sprite("ChordOutline");
        let time = note.time_offset.max(self.base.current_time);
        let z = -(time * self.base.time_scale);
        let start_x = self.get_fret_position((note.hand_fret - 1) as f32);
        let end_x = self.get_fret_position((note.hand_fret + 3) as f32);
        let end_height =
            if time == self.base.current_time || note.time_offset > self.base.current_time {
                self.get_string_height(self.num_strings as f32)
            } else {
                self.get_string_height(2.0)
            };
        let alpha = if note.has_technique(SongNoteTechnique::ACCENT) {
            1.0
        } else {
            0.32
        };
        self.base.base.push_world_nine_patch(
            vertices,
            Vector3::new(start_x, 0.0, z),
            Vector3::new(start_x, end_height, z),
            Vector3::new(end_x, end_height, z),
            Vector3::new(end_x, 0.0, z),
            [1.0, 1.0, 1.0, alpha],
            sprite,
        );

        if (note.has_technique(SongNoteTechnique::PALM_MUTE)
            || note.has_technique(SongNoteTechnique::FRET_HAND_MUTE))
            && time > self.base.current_time
        {
            let mute_sprite = sprites.sprite(if note.has_technique(SongNoteTechnique::PALM_MUTE) {
                "NotePalmMute"
            } else {
                "NoteMute"
            });
            self.push_modifier(
                vertices,
                note.hand_fret as f32 + 1.0,
                time,
                self.get_string_height(0.5),
                mute_sprite,
            );
        }

        if let Some(name) = chord.name.as_deref().filter(|name| !name.is_empty()) {
            self.push_vertical_text(
                vertices,
                name,
                note.hand_fret as f32 - 1.02,
                self.get_string_height(self.num_strings as f32 - 1.0),
                time,
                [1.0, 1.0, 1.0, 1.0],
                0.09,
                true,
                sprites,
            );
        }
    }

    fn push_finger_overlays(
        &self,
        vertices: &mut Vec<SceneVertex>,
        note: &SongNote,
        chord: &SongChord,
        sprites: &SpriteLibrary,
    ) {
        let sprite = sprites.sprite("FingerOutline");
        for (string_index, fret) in chord.frets.iter().enumerate() {
            if *fret < 0 || string_index >= self.num_strings as usize {
                continue;
            }
            let draw_fret = if *fret == 0 {
                note.hand_fret as f32 + 1.5
            } else {
                *fret as f32
            };
            let string_offset = self.get_string_offset(string_index as i32).max(0) as f32;
            let time = self.base.current_time.max(note.time_offset);
            self.push_modifier(
                vertices,
                draw_fret - 0.5,
                time,
                self.get_string_height(string_offset),
                sprite,
            );

            if let Some(finger) = chord
                .fingers
                .get(string_index)
                .copied()
                .filter(|finger| *finger > 0)
            {
                let finger_text = finger.to_string();
                self.push_vertical_text(
                    vertices,
                    finger_text.as_str(),
                    draw_fret - 0.5,
                    self.get_string_height(string_offset),
                    time,
                    [1.0, 1.0, 1.0, 1.0],
                    0.05,
                    false,
                    sprites,
                );
            }
        }
    }

    fn push_modifier(
        &self,
        vertices: &mut Vec<SceneVertex>,
        fret_center: f32,
        time: f32,
        height: f32,
        sprite: SpriteRegion,
    ) {
        let x = self.get_fret_position(fret_center);
        let z = -(time * self.base.time_scale);
        let half_width = sprite.width as f32 * 0.08;
        let half_height = sprite.height as f32 * 0.08;
        self.base.base.push_world_quad(
            vertices,
            Vector3::new(x - half_width, height - half_height, z),
            Vector3::new(x - half_width, height + half_height, z),
            Vector3::new(x + half_width, height + half_height, z),
            Vector3::new(x + half_width, height - half_height, z),
            [1.0, 1.0, 1.0, 1.0],
            sprite,
        );
    }

    fn push_note_shadow(
        &self,
        vertices: &mut Vec<SceneVertex>,
        draw_fret: f32,
        note_head_time: f32,
        string_offset: f32,
        sprites: &SpriteLibrary,
    ) {
        let horizontal = sprites.sprite("HorizontalFretLine");
        self.base.push_horizontal_line(
            vertices,
            self.get_fret_position(draw_fret - 1.0),
            self.get_fret_position(draw_fret),
            note_head_time,
            0.0,
            self.base.white_half_alpha,
            horizontal,
            0.08,
        );

        for prev_string in 0..string_offset as i32 {
            self.base.push_horizontal_line(
                vertices,
                self.get_fret_position(draw_fret - 0.6),
                self.get_fret_position(draw_fret - 0.4),
                note_head_time,
                self.get_string_height(prev_string as f32),
                self.base.white_half_alpha,
                horizontal,
                0.04,
            );
        }
    }

    fn push_slide_tail(
        &self,
        vertices: &mut Vec<SceneVertex>,
        note: &SongNote,
        draw_fret: f32,
        start_time: f32,
        end_time: f32,
        string_height: f32,
        sprite: SpriteRegion,
        color: [f32; 4],
    ) {
        self.push_image_trail(
            vertices,
            sprite,
            color,
            0.03,
            &[
                Vector3::new(draw_fret - 0.5, string_height, start_time),
                Vector3::new(note.slide_fret as f32 - 0.5, string_height, end_time),
            ],
        );
    }

    fn push_vibrato_trail(
        &self,
        vertices: &mut Vec<SceneVertex>,
        draw_fret: f32,
        start_time: f32,
        end_time: f32,
        string_height: f32,
        sprite: SpriteRegion,
        color: [f32; 4],
    ) {
        if end_time <= self.base.current_time || end_time <= start_time {
            return;
        }

        let mut last_time = start_time.max(self.base.current_time);
        let mut last_height = string_height;
        let num_points = ((end_time - start_time) / 0.02).max(1.0) as i32;

        for point_index in 1..=num_points {
            let progress = point_index as f32 / num_points as f32;
            let time = lerp(start_time, end_time, progress);
            if time < self.base.current_time {
                last_time = self.base.current_time;
                continue;
            }

            let height = string_height + ((time - start_time) * 50.0).sin();
            self.push_image_trail(
                vertices,
                sprite,
                color,
                0.03,
                &[
                    Vector3::new(draw_fret - 0.5, last_height, last_time),
                    Vector3::new(draw_fret - 0.5, height, time),
                ],
            );

            last_height = height;
            last_time = time;
        }
    }

    fn push_bend_trail(
        &self,
        vertices: &mut Vec<SceneVertex>,
        note: &SongNote,
        string_offset: f32,
        end_time: f32,
        sprite: SpriteRegion,
        color: [f32; 4],
    ) {
        let Some(cents_offsets) = note.cents_offsets.as_ref() else {
            return;
        };
        if end_time <= self.base.current_time {
            return;
        }

        let fret_center = note.fret as f32 - 0.5;
        let base_height = self.get_string_height(string_offset);
        let mut last_time = note.time_offset;
        let mut last_height = base_height;

        for offset in cents_offsets {
            let height =
                base_height + self.get_cents_height_offset(string_offset, offset.cents as f32);
            if offset.time_offset >= self.base.current_time && offset.time_offset > last_time {
                if last_time < self.base.current_time {
                    let duration = (offset.time_offset - last_time).max(f32::EPSILON);
                    let progress =
                        ((self.base.current_time - last_time) / duration).clamp(0.0, 1.0);
                    last_height = lerp(last_height, height, progress);
                    last_time = self.base.current_time;
                }

                let segment_end_time = offset.time_offset.min(end_time);
                let segment_end_height = if offset.time_offset > end_time {
                    let duration = (offset.time_offset - last_time).max(f32::EPSILON);
                    let progress = ((end_time - last_time) / duration).clamp(0.0, 1.0);
                    lerp(last_height, height, progress)
                } else {
                    height
                };

                self.push_image_trail(
                    vertices,
                    sprite,
                    color,
                    0.03,
                    &[
                        Vector3::new(fret_center, last_height, last_time),
                        Vector3::new(fret_center, segment_end_height, segment_end_time),
                    ],
                );
            }

            last_height = height;
            last_time = offset.time_offset;
            if last_time >= end_time {
                return;
            }
        }

        if last_time < end_time {
            self.push_image_trail(
                vertices,
                sprite,
                color,
                0.03,
                &[
                    Vector3::new(
                        fret_center,
                        last_height,
                        last_time.max(self.base.current_time),
                    ),
                    Vector3::new(fret_center, last_height, end_time),
                ],
            );
        }
    }

    fn push_image_trail(
        &self,
        vertices: &mut Vec<SceneVertex>,
        sprite: SpriteRegion,
        color: [f32; 4],
        image_scale: f32,
        trail_points: &[Vector3<f32>],
    ) {
        let half_width = sprite.width as f32 * image_scale;

        for points in trail_points.windows(2) {
            let start = points[0];
            let end = points[1];
            let start_x = self.get_fret_position(start.x);
            let end_x = self.get_fret_position(end.x);
            let start_z = -(start.z * self.base.time_scale);
            let end_z = -(end.z * self.base.time_scale);

            self.base.base.push_world_quad(
                vertices,
                Vector3::new(start_x - half_width, start.y, start_z),
                Vector3::new(end_x - half_width, end.y, end_z),
                Vector3::new(end_x + half_width, end.y, end_z),
                Vector3::new(start_x + half_width, start.y, start_z),
                color,
                sprite,
            );
        }
    }

    fn push_fret_timeline(
        &self,
        vertices: &mut Vec<SceneVertex>,
        fret: f32,
        start_time: f32,
        end_time: f32,
        color: [f32; 4],
        sprite: SpriteRegion,
        image_scale: f32,
    ) {
        let x = self.get_fret_position(fret);
        let start_z = -(start_time * self.base.time_scale);
        let end_z = -(end_time * self.base.time_scale);
        let half_width = sprite.width as f32 * image_scale;
        self.base.base.push_world_quad(
            vertices,
            Vector3::new(x - half_width, 0.0, start_z),
            Vector3::new(x - half_width, 0.0, end_z),
            Vector3::new(x + half_width, 0.0, end_z),
            Vector3::new(x + half_width, 0.0, start_z),
            color,
            sprite,
        );
    }

    fn push_fret_marker(
        &self,
        vertices: &mut Vec<SceneVertex>,
        fret_center: f32,
        time: f32,
        start_height: f32,
        end_height: f32,
        color: [f32; 4],
        sprite: SpriteRegion,
        image_scale: f32,
    ) {
        let x = self.get_fret_position(fret_center);
        let z = -(time * self.base.time_scale);
        let half_width = sprite.width as f32 * image_scale;
        self.base.base.push_world_quad(
            vertices,
            Vector3::new(x - half_width, start_height, z),
            Vector3::new(x - half_width, end_height, z),
            Vector3::new(x + half_width, end_height, z),
            Vector3::new(x + half_width, start_height, z),
            color,
            sprite,
        );
    }

    fn push_fret_tail(
        &self,
        vertices: &mut Vec<SceneVertex>,
        _note: &SongNote,
        draw_fret: f32,
        start_time: f32,
        end_time: f32,
        string_height: f32,
        sprite: SpriteRegion,
        color: [f32; 4],
    ) {
        let fret_center = draw_fret - 0.5;
        let x = self.get_fret_position(fret_center);
        let half_width = sprite.width as f32 * 0.03;
        let start_z = -(start_time * self.base.time_scale);
        let end_z = -(end_time * self.base.time_scale);
        self.base.base.push_world_quad(
            vertices,
            Vector3::new(x - half_width, string_height, start_z),
            Vector3::new(x - half_width, string_height, end_z),
            Vector3::new(x + half_width, string_height, end_z),
            Vector3::new(x + half_width, string_height, start_z),
            color,
            sprite,
        );
    }

    fn push_fret_head(
        &self,
        vertices: &mut Vec<SceneVertex>,
        note: &SongNote,
        draw_fret: f32,
        time: f32,
        string_height: f32,
        sprite: SpriteRegion,
        color: [f32; 4],
    ) {
        let z = -(time * self.base.time_scale);
        let half_height = sprite.height as f32 * if note.fret == 0 { 0.04 } else { 0.08 };

        if note.fret == 0 {
            let start_x = self.get_fret_position((note.hand_fret - 1) as f32);
            let end_x = self.get_fret_position((note.hand_fret + 3) as f32);
            self.base.base.push_world_quad(
                vertices,
                Vector3::new(start_x, string_height - half_height, z),
                Vector3::new(start_x, string_height + half_height, z),
                Vector3::new(end_x, string_height + half_height, z),
                Vector3::new(end_x, string_height - half_height, z),
                color,
                sprite,
            );
        } else {
            let center_x = self.get_fret_position(draw_fret - 0.5);
            let half_width = sprite.width as f32 * 0.08;
            self.base.base.push_world_quad(
                vertices,
                Vector3::new(center_x - half_width, string_height - half_height, z),
                Vector3::new(center_x - half_width, string_height + half_height, z),
                Vector3::new(center_x + half_width, string_height + half_height, z),
                Vector3::new(center_x + half_width, string_height - half_height, z),
                color,
                sprite,
            );
        }
    }

    fn push_vertical_text(
        &self,
        vertices: &mut Vec<SceneVertex>,
        text: &str,
        fret_center: f32,
        vertical_center: f32,
        time_center: f32,
        color: [f32; 4],
        image_scale: f32,
        right_align: bool,
        sprites: &SpriteLibrary,
    ) {
        let Some(font) = sprites.font("LargeFont") else {
            return;
        };

        let fret_center = self.get_fret_position(fret_center);
        let (text_width, text_height) = font.measure_text(text, image_scale);
        let mut x = if right_align {
            fret_center - text_width
        } else {
            fret_center - (text_width * 0.5)
        };
        let y = vertical_center - (text_height * 0.5);
        let z = -(time_center * self.base.time_scale);

        let characters: Vec<char> = if self.base.lefty_mode {
            text.chars().rev().collect()
        } else {
            text.chars().collect()
        };

        for character in characters {
            let Some(glyph) = font.glyph(character) else {
                continue;
            };

            let glyph_width = glyph.region.width as f32 * image_scale;
            let glyph_height = glyph.region.height as f32 * image_scale;
            let glyph_region = if self.base.lefty_mode {
                SpriteRegion {
                    u0: glyph.region.u1,
                    u1: glyph.region.u0,
                    ..glyph.region
                }
            } else {
                glyph.region
            };

            self.base.base.push_world_quad(
                vertices,
                Vector3::new(x, y, z),
                Vector3::new(x, y + glyph_height, z),
                Vector3::new(x + glyph_width, y + glyph_height, z),
                Vector3::new(x + glyph_width, y, z),
                color,
                glyph_region,
            );

            x += (glyph.region.width as f32 + font.spacing) * image_scale;
        }
    }

    fn get_string_offset(&self, string_index: i32) -> i32 {
        if self.base.lefty_mode {
            self.num_strings - string_index - 1
        } else {
            string_index
        }
    }

    fn get_string_color(&self, string_index: usize) -> [f32; 4] {
        let index =
            (string_index + self.string_color_offset as usize).min(self.string_colors.len() - 1);
        self.string_colors[index]
    }

    fn note_head_sprite_name(&self, string_index: usize) -> String {
        format!(
            "Guitar{}",
            self.string_color_names[string_index.min(self.string_color_names.len() - 1)]
        )
    }

    fn note_trail_sprite_name(&self, string_index: usize) -> String {
        format!(
            "NoteTrail{}",
            self.string_color_names[string_index.min(self.string_color_names.len() - 1)]
        )
    }

    fn note_modifier_sprite(
        &self,
        note: &SongNote,
        sprites: &SpriteLibrary,
    ) -> Option<SpriteRegion> {
        let name = if note.has_technique(SongNoteTechnique::HAMMER_ON) {
            Some("NoteHammerOn")
        } else if note.has_technique(SongNoteTechnique::PULL_OFF) {
            Some("NotePullOff")
        } else if note.has_technique(SongNoteTechnique::FRET_HAND_MUTE) {
            Some("NoteMute")
        } else if note.has_technique(SongNoteTechnique::PALM_MUTE) {
            Some("NotePalmMute")
        } else if note.has_technique(SongNoteTechnique::HARMONIC) {
            Some("NoteHarmonic")
        } else if note.has_technique(SongNoteTechnique::PINCH_HARMONIC) {
            Some("NotePinchHarmonic")
        } else {
            None
        }?;

        Some(sprites.sprite(name))
    }

    fn get_draw_fret(&self, note: &SongNote, ref_time: f32) -> f32 {
        if note.fret == 0 {
            return note.hand_fret as f32 + 1.5;
        }

        if note.has_technique(SongNoteTechnique::SLIDE)
            && note.slide_fret >= 0
            && note.time_length > 0.0
            && ref_time > note.time_offset
        {
            let progress = ((ref_time - note.time_offset) / note.time_length).clamp(0.0, 1.0);
            return lerp(note.fret as f32, note.slide_fret as f32, progress);
        }

        note.fret as f32
    }

    fn get_note_color(&self, note: &SongNote, is_ghost: bool) -> [f32; 4] {
        let mut color = self.get_string_color(note.string.max(0) as usize);
        if note.has_technique(SongNoteTechnique::ACCENT) {
            color[0] = lerp(color[0], 1.0, 0.75);
            color[1] = lerp(color[1], 1.0, 0.75);
            color[2] = lerp(color[2], 1.0, 0.75);
        }

        if note.has_technique(SongNoteTechnique::PALM_MUTE)
            || note.has_technique(SongNoteTechnique::FRET_HAND_MUTE)
        {
            color[0] *= 0.55;
            color[1] *= 0.55;
            color[2] *= 0.55;
        }

        if is_ghost {
            color[3] = 0.25;
        }

        color
    }

    fn get_note_head_height(&self, note: &SongNote, string_offset: f32) -> f32 {
        let mut height = self.get_string_height(string_offset);
        let bend_cents = self.get_bend_cents(note, self.base.current_time.max(note.time_offset));
        if bend_cents != 0.0 {
            height += self.get_cents_height_offset(string_offset, bend_cents);
        }
        height
    }

    fn get_cents_height_offset(&self, string_offset: f32, cents: f32) -> f32 {
        if string_offset < 2.0 {
            cents / 30.0
        } else {
            -(cents / 30.0)
        }
    }

    fn get_bend_cents(&self, note: &SongNote, ref_time: f32) -> f32 {
        let Some(cents_offsets) = note.cents_offsets.as_ref() else {
            return 0.0;
        };
        let Some(first) = cents_offsets.first() else {
            return 0.0;
        };

        let ref_time = ref_time.max(note.time_offset);
        if cents_offsets.len() == 1 {
            return if (first.time_offset - note.time_offset).abs() < f32::EPSILON {
                first.cents as f32
            } else {
                0.0
            };
        }

        if ref_time <= first.time_offset {
            return first.cents as f32;
        }

        for window in cents_offsets.windows(2) {
            let start = &window[0];
            let end = &window[1];
            if ref_time <= end.time_offset {
                let duration = (end.time_offset - start.time_offset).max(f32::EPSILON);
                let progress = ((ref_time - start.time_offset) / duration).clamp(0.0, 1.0);
                return lerp(start.cents as f32, end.cents as f32, progress);
            }
        }

        cents_offsets
            .last()
            .map(|offset| offset.cents as f32)
            .unwrap_or(0.0)
    }
}

impl Default for FretPlayerScene3D {
    fn default() -> Self {
        Self::new(SongPlayer::shared_demo_for(SongInstrumentType::LeadGuitar))
    }
}

#[derive(Debug, Clone)]
pub struct KeysPlayerScene3D {
    pub base: ChartScene3D,
    min_key: i32,
    max_key: i32,
    target_camera_distance: f32,
    camera_distance: f32,
    position_key: f32,
    start_note_position: i32,
    scale_white_black: [i32; 12],
    scale_offsets: [f32; 12],
}

impl KeysPlayerScene3D {
    pub fn new(player: SharedSongPlayer) -> Self {
        Self {
            base: ChartScene3D::new(Some(player)),
            min_key: 48,
            max_key: 72,
            target_camera_distance: 64.0,
            camera_distance: 70.0,
            position_key: 60.0,
            start_note_position: 0,
            scale_white_black: [0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0],
            scale_offsets: [0.0, 0.5, 1.0, 1.5, 2.0, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5, 6.0],
        }
    }

    pub fn get_key_position(&self, key: f32) -> f32 {
        let int_key = key.floor() as i32;
        if (key - int_key as f32).abs() < f32::EPSILON {
            let octave = (int_key - self.min_key) / 12;
            let note_in_octave = (int_key - self.min_key) % 12;
            (self.scale_offsets[note_in_octave as usize] + octave as f32 * 7.0) * 8.0
        } else {
            let pos = self.get_key_position(int_key as f32);
            let frac = key - int_key as f32;
            pos + frac * 8.0
        }
    }

    pub fn build_vertices(
        &mut self,
        current_time: f32,
        viewport_width: u32,
        viewport_height: u32,
        sprites: &SpriteLibrary,
    ) -> Vec<SceneVertex> {
        self.base
            .begin_frame(current_time, viewport_width, viewport_height);
        self.base.highway_start_x = self.get_key_position(self.min_key as f32);
        self.base.highway_end_x = self.get_key_position(self.max_key as f32 + 2.0);
        self.update_camera();

        let mut vertices = Vec::with_capacity(1024);
        let vertical = sprites.sprite("VerticalFretLine");
        for key in self.min_key..=(self.max_key + 2) {
            if self.scale_white_black[(key - self.min_key) as usize % 12] == 0 {
                self.push_key_timeline(
                    &mut vertices,
                    key as f32,
                    self.base.start_time,
                    self.base.end_time,
                    [1.0, 1.0, 1.0, 0.35],
                    vertical,
                    0.03,
                );
            }
        }

        self.base.push_beat_lines(&mut vertices, sprites, 0.0, 0.08);

        if let Some(player) = self.base.player.as_ref() {
            let notes = player
                .lock()
                .ok()
                .map(|player| player.get_keyboard_notes())
                .unwrap_or_default();
            if !notes.is_empty() {
                self.start_note_position = self.base.get_start_note(
                    self.base.current_time,
                    0.15,
                    self.start_note_position,
                    &notes,
                );
                let end_position =
                    self.base
                        .get_end_note(self.start_note_position, self.base.end_time, &notes);

                for note in notes
                    .iter()
                    .take(end_position.max(0) as usize + 1)
                    .skip(self.start_note_position.max(0) as usize)
                {
                    if note.note < self.min_key
                        || note.note > self.max_key
                        || note.time_offset > self.base.end_time
                    {
                        continue;
                    }

                    let is_white =
                        self.scale_white_black[(note.note - self.min_key) as usize % 12] == 0;
                    let sprite = sprites.sprite(if is_white {
                        "NoteTrailWhite"
                    } else {
                        "NoteTrailBlack"
                    });
                    self.push_key_tail(
                        &mut vertices,
                        note.note as f32 + 0.5,
                        note.time_offset.max(self.base.current_time),
                        note.end_time,
                        0.0,
                        [1.0, 1.0, 1.0, 1.0],
                        sprite,
                        0.06,
                    );
                }
            }
        }

        vertices
    }

    fn update_camera(&mut self) {
        let key_dist = (self.max_key - self.min_key - 12).max(0);
        self.target_camera_distance = 60.0 + key_dist.max(4) as f32 * 3.0;
        let target_position_key = (self.max_key + self.min_key) as f32 * 0.5;
        self.position_key = lerp(self.position_key, target_position_key, 0.01);
        self.camera_distance = lerp(self.camera_distance, self.target_camera_distance, 0.01);
        self.base.base.camera.position = Vector3::new(
            self.get_key_position(self.position_key),
            50.0,
            -(self.base.current_time * self.base.time_scale) + self.camera_distance,
        );
        self.base.base.camera.set_look_at(Vector3::new(
            self.get_key_position(self.position_key),
            0.0,
            self.base.base.camera.position.z
                - (self.base.note_display_seconds * self.base.time_scale) * 0.3,
        ));
    }

    fn push_key_timeline(
        &self,
        vertices: &mut Vec<SceneVertex>,
        key_center: f32,
        start_time: f32,
        end_time: f32,
        color: [f32; 4],
        sprite: SpriteRegion,
        image_scale: f32,
    ) {
        let x = self.get_key_position(key_center);
        let start_z = -(start_time * self.base.time_scale);
        let end_z = -(end_time * self.base.time_scale);
        let half_width = sprite.width as f32 * image_scale;
        self.base.base.push_world_quad(
            vertices,
            Vector3::new(x - half_width, 0.0, start_z),
            Vector3::new(x - half_width, 0.0, end_z),
            Vector3::new(x + half_width, 0.0, end_z),
            Vector3::new(x + half_width, 0.0, start_z),
            color,
            sprite,
        );
    }

    fn push_key_tail(
        &self,
        vertices: &mut Vec<SceneVertex>,
        key_center: f32,
        start_time: f32,
        end_time: f32,
        height_offset: f32,
        color: [f32; 4],
        sprite: SpriteRegion,
        image_scale: f32,
    ) {
        let x = self.get_key_position(key_center);
        let half_width = sprite.width as f32 * image_scale;
        let start_z = -(start_time * self.base.time_scale);
        let end_z = -(end_time * self.base.time_scale);
        self.base.base.push_world_quad(
            vertices,
            Vector3::new(x - half_width, height_offset, start_z),
            Vector3::new(x - half_width, height_offset, end_z),
            Vector3::new(x + half_width, height_offset, end_z),
            Vector3::new(x + half_width, height_offset, start_z),
            color,
            sprite,
        );
    }
}

impl Default for KeysPlayerScene3D {
    fn default() -> Self {
        Self::new(SongPlayer::shared_demo_for(SongInstrumentType::Keys))
    }
}

#[derive(Debug, Clone)]
pub enum PlayerScene3D {
    Fret(FretPlayerScene3D),
    Drum(DrumPlayerScene3D),
    Keys(KeysPlayerScene3D),
}

impl PlayerScene3D {
    pub fn name(&self) -> &'static str {
        match self {
            PlayerScene3D::Fret(_) => "Fretboard",
            PlayerScene3D::Drum(_) => "Drums",
            PlayerScene3D::Keys(_) => "Keys",
        }
    }

    pub fn current_time_seconds(&self) -> f32 {
        match self {
            PlayerScene3D::Fret(scene) => scene.base.current_playback_seconds(),
            PlayerScene3D::Drum(scene) => scene.base.current_playback_seconds(),
            PlayerScene3D::Keys(scene) => scene.base.current_playback_seconds(),
        }
    }

    pub fn player_handle(&self) -> Option<SharedSongPlayer> {
        match self {
            PlayerScene3D::Fret(scene) => scene.base.player_handle(),
            PlayerScene3D::Drum(scene) => scene.base.player_handle(),
            PlayerScene3D::Keys(scene) => scene.base.player_handle(),
        }
    }

    pub fn build_vertices(
        &mut self,
        current_time: f32,
        viewport_width: u32,
        viewport_height: u32,
        sprites: &SpriteLibrary,
    ) -> Vec<SceneVertex> {
        match self {
            PlayerScene3D::Fret(scene) => {
                scene.build_vertices(current_time, viewport_width, viewport_height, sprites)
            }
            PlayerScene3D::Drum(scene) => {
                scene.build_vertices(current_time, viewport_width, viewport_height, sprites)
            }
            PlayerScene3D::Keys(scene) => {
                scene.build_vertices(current_time, viewport_width, viewport_height, sprites)
            }
        }
    }
}

fn fret_position(fret: f32) -> f32 {
    const SCALE_LENGTH: f32 = 300.0;
    SCALE_LENGTH - (SCALE_LENGTH / 2.0_f32.powf(fret / 12.0))
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
