use egui_wgpu::wgpu::{self, Device, Queue, RenderPass, SurfaceConfiguration};
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, Style,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer as GlyphonTextRenderer, Viewport,
    Weight,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use winit::window::Window;

#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub font_family: String,
    pub font_size: f32,
    pub line_height: f32,
    pub color: Color,
    pub weight: Weight,
    pub style: Style,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: "DejaVu Sans".to_string(),
            font_size: 16.0,
            line_height: 20.0,
            color: Color::rgb(255, 255, 255),
            weight: Weight::NORMAL,
            style: Style::Normal,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextPosition {
    pub x: f32,
    pub y: f32,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
}

impl Default for TextPosition {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            max_width: None,
            max_height: None,
        }
    }
}

#[derive(Debug)]
pub struct TextBuffer {
    pub buffer: Buffer,
    pub style: TextStyle,
    pub position: TextPosition,
    pub scale: f32,
    pub visible: bool,
    pub text_content: String,
}

pub struct TextRenderer {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub viewport: Viewport,
    pub atlas: TextAtlas,
    pub glyph_renderer: GlyphonTextRenderer,
    pub text_buffers: HashMap<String, TextBuffer>,
    pub window_size: winit::dpi::PhysicalSize<u32>,
    pub loaded_fonts: Vec<String>,
}

impl TextRenderer {
    pub fn new(
        device: &Device,
        queue: &Queue,
        surface_format: wgpu::TextureFormat,
        window: &Window,
    ) -> Self {
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, surface_format);
        let glyph_renderer =
            GlyphonTextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);

        let size = window.inner_size();

        let mut renderer = Self {
            font_system,
            swash_cache,
            viewport,
            atlas,
            glyph_renderer,
            text_buffers: HashMap::new(),
            window_size: size,
            loaded_fonts: Vec::new(),
        };

        // Try to load the custom font, but don't fail if it doesn't exist
        match renderer.load_font(
            "fonts/HankenGrotesk/HankenGrotesk-Medium.ttf",
            "HankenGrotesk",
        ) {
            Ok(_) => println!("Successfully loaded HankenGrotesk font"),
            Err(e) => {
                println!(
                    "Failed to load HankenGrotesk font: {}. Using system fonts.",
                    e
                );
            }
        }

        renderer
    }

    /// Load a font from a file path and register it with a name
    pub fn load_font(&mut self, font_path: &str, font_name: &str) -> Result<(), std::io::Error> {
        let font_data = fs::read(Path::new(font_path))?;
        self.font_system.db_mut().load_font_data(font_data);
        self.loaded_fonts.push(font_name.to_string());
        println!("Loaded font: {} from {}", font_name, font_path);
        Ok(())
    }

    /// Create a new text buffer with the given ID, text, style, and position
    pub fn create_text_buffer(
        &mut self,
        id: &str,
        text: &str,
        style: Option<TextStyle>,
        position: Option<TextPosition>,
    ) {
        let mut style = style.unwrap_or_default();
        let position = position.unwrap_or_default();

        // If the requested font isn't loaded, fall back to a system font
        if !self.loaded_fonts.contains(&style.font_family) && style.font_family == "HankenGrotesk" {
            style.font_family = "DejaVu Sans".to_string();
        }

        let metrics = Metrics::new(style.font_size, style.line_height);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        // Set buffer size based on position constraints or window size
        let width = position.max_width.unwrap_or(self.window_size.width as f32);
        let height = position
            .max_height
            .unwrap_or(self.window_size.height as f32);

        buffer.set_size(&mut self.font_system, Some(width), Some(height));

        let attrs = Attrs::new()
            .family(Family::Name(&style.font_family))
            .weight(style.weight)
            .style(style.style);

        buffer.set_text(&mut self.font_system, text, attrs, Shaping::Advanced);
        buffer.shape_until_scroll(&mut self.font_system, false);

        let text_buffer = TextBuffer {
            buffer,
            style,
            position,
            scale: 1.0,
            visible: true,
            text_content: text.to_string(),
        };

        self.text_buffers.insert(id.to_string(), text_buffer);
    }

    /// Update the style of an existing buffer
    pub fn update_style(&mut self, id: &str, mut style: TextStyle) -> Result<(), String> {
        let text_buffer = self
            .text_buffers
            .get_mut(id)
            .ok_or_else(|| format!("Text buffer '{}' not found", id))?;

        // If the requested font isn't loaded, fall back to a system font
        if !self.loaded_fonts.contains(&style.font_family) && style.font_family == "HankenGrotesk" {
            style.font_family = "DejaVu Sans".to_string();
        }

        // Update metrics if font size or line height changed
        if text_buffer.style.font_size != style.font_size
            || text_buffer.style.line_height != style.line_height
        {
            let metrics = Metrics::new(style.font_size, style.line_height);
            text_buffer
                .buffer
                .set_metrics(&mut self.font_system, metrics);
        }

        text_buffer.style = style;

        // Re-apply text with new attributes using stored content
        let attrs = Attrs::new()
            .family(Family::Name(&text_buffer.style.font_family))
            .weight(text_buffer.style.weight)
            .style(text_buffer.style.style);

        text_buffer.buffer.set_text(
            &mut self.font_system,
            &text_buffer.text_content,
            attrs,
            Shaping::Advanced,
        );
        text_buffer
            .buffer
            .shape_until_scroll(&mut self.font_system, false);
        Ok(())
    }

    /// Update the position of an existing buffer
    pub fn update_position(&mut self, id: &str, position: TextPosition) -> Result<(), String> {
        let text_buffer = self
            .text_buffers
            .get_mut(id)
            .ok_or_else(|| format!("Text buffer '{}' not found", id))?;

        // Update buffer size if max dimensions changed
        if text_buffer.position.max_width != position.max_width
            || text_buffer.position.max_height != position.max_height
        {
            let width = position.max_width.unwrap_or(self.window_size.width as f32);
            let height = position
                .max_height
                .unwrap_or(self.window_size.height as f32);
            text_buffer
                .buffer
                .set_size(&mut self.font_system, Some(width), Some(height));
        }

        text_buffer.position = position;
        Ok(())
    }

    pub fn resize(&mut self, queue: &Queue, resolution: Resolution) {
        self.viewport.update(queue, resolution);
    }

    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        _surface_config: &SurfaceConfiguration,
    ) -> Result<(), glyphon::PrepareError> {
        let mut text_areas = Vec::new();

        for text_buffer in self.text_buffers.values() {
            if !text_buffer.visible {
                continue;
            }

            let bounds = TextBounds {
                left: text_buffer.position.x as i32,
                top: text_buffer.position.y as i32,
                right: (text_buffer.position.x
                    + text_buffer
                        .position
                        .max_width
                        .unwrap_or(self.window_size.width as f32)) as i32,
                bottom: (text_buffer.position.y
                    + text_buffer
                        .position
                        .max_height
                        .unwrap_or(self.window_size.height as f32)) as i32,
            };

            let text_area = TextArea {
                buffer: &text_buffer.buffer,
                left: text_buffer.position.x,
                top: text_buffer.position.y,
                scale: text_buffer.scale,
                bounds,
                default_color: text_buffer.style.color,
                custom_glyphs: &[],
            };

            text_areas.push(text_area);
        }

        self.glyph_renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            text_areas,
            &mut self.swash_cache,
        )
    }

    pub fn render(&mut self, render_pass: &mut RenderPass) -> Result<(), glyphon::RenderError> {
        self.glyph_renderer
            .render(&self.atlas, &self.viewport, render_pass)
    }

    pub fn measure_text(&mut self, text: &str, style: &TextStyle) -> (f32, f32, f32) {
        let metrics = Metrics::new(style.font_size, style.line_height);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        let attrs = Attrs::new()
            .family(Family::Name(&style.font_family))
            .weight(style.weight)
            .style(style.style);

        buffer.set_text(&mut self.font_system, text, attrs, Shaping::Advanced);
        buffer.shape_until_scroll(&mut self.font_system, false);

        // Calculate text dimensions from layout runs
        let mut min_x = f32::MAX;
        let mut max_x: f32 = 0.0;
        let mut height: f32 = 0.0;

        for run in buffer.layout_runs() {
            if let Some(first_glyph) = run.glyphs.first() {
                min_x = min_x.min(first_glyph.x);
            }
            if let Some(last_glyph) = run.glyphs.last() {
                max_x = max_x.max(last_glyph.x + last_glyph.w);
            }
            height += run.line_height;
        }

        // If no runs, estimate based on text length and font size
        if min_x == f32::MAX && !text.is_empty() {
            min_x = 0.0;
            max_x = text.len() as f32 * style.font_size * 0.6;
            height = style.line_height;
        }

        let width = max_x - min_x;
        (min_x, width, height)
    }
}
