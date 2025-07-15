use crate::game;
use crate::game::{GameUIManager, PauseMenuState};
use crate::pause_menu::{PauseMenu, PauseMenuAction};
use crate::text::TextRenderer;
use egui_wgpu::wgpu;
use egui_wgpu::wgpu::SurfaceError;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

pub struct AppState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
    pub pause_menu: PauseMenu,
    pub text_renderer: TextRenderer,
    pub game_ui: GameUIManager,
    pub pause_menu_state: PauseMenuState,
}

impl AppState {
    async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        window: &Window,
        width: u32,
        height: u32,
    ) -> Self {
        let power_pref = wgpu::PowerPreference::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: power_pref,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let features = wgpu::Features::empty();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: features,
                    required_limits: Default::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let selected_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let swapchain_format = swapchain_capabilities
            .formats
            .iter()
            .find(|d| **d == selected_format)
            .expect("failed to select proper surface texture format!");

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *swapchain_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 0,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        let pause_menu = PauseMenu::new(&device, &queue, surface_config.format, window);
        let mut text_renderer = TextRenderer::new(&device, &queue, surface_config.format, window);
        let mut game_ui = GameUIManager::new();
        game_ui.start_timer(None);
        game::initialize_game_ui(&mut text_renderer, &game_ui, window);
        let pause_menu_state = if pause_menu.is_visible() {
            PauseMenuState::Open
        } else {
            PauseMenuState::Closed
        };
        Self {
            device,
            queue,
            surface,
            surface_config,
            pause_menu,
            text_renderer,
            game_ui,
            pause_menu_state,
        }
    }

    fn resize_surface(&mut self, width: u32, height: u32, window: &Window) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
        let resolution = glyphon::Resolution { width, height };
        self.pause_menu.resize(&self.queue, resolution);
        self.text_renderer.resize(&self.queue, resolution);
        // Re-initialize game UI text positions with the actual window
        game::initialize_game_ui(&mut self.text_renderer, &self.game_ui, window);
    }
}

pub struct App {
    instance: wgpu::Instance,
    state: Option<AppState>,
    window: Option<Arc<Window>>,
}

impl App {
    pub fn new() -> Self {
        let instance = egui_wgpu::wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        Self {
            instance,
            state: None,
            window: None,
        }
    }

    async fn set_window(&mut self, window: Window) {
        let window = Arc::new(window);
        let initial_width = 1360;
        let initial_height = 768;

        let _ = window.request_inner_size(PhysicalSize::new(initial_width, initial_height));

        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("Failed to create surface!");

        let state = AppState::new(
            &self.instance,
            surface,
            &window,
            initial_width,
            initial_width,
        )
        .await;

        self.window.get_or_insert(window);
        self.state.get_or_insert(state);
    }

    fn handle_resized(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            if let Some(window) = self.window.as_ref() {
                self.state
                    .as_mut()
                    .unwrap()
                    .resize_surface(width, height, window);
            }
        }
    }

    fn handle_redraw(&mut self) {
        // Handle minimizing window
        if let Some(window) = self.window.as_ref() {
            if let Some(min) = window.is_minimized() {
                if min {
                    return;
                }
            }
        }

        let state = self.state.as_mut().unwrap();

        let surface_texture = state.surface.get_current_texture();

        match surface_texture {
            Err(SurfaceError::Outdated) => {
                return;
            }
            Err(_) => {
                surface_texture.expect("Failed to acquire next swap chain texture");
                return;
            }
            Ok(_) => {}
        };

        let surface_texture = surface_texture.unwrap();

        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Clear the screen with a muted blue background
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.18, // muted blue
                            g: 0.24,
                            b: 0.32,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                label: Some("clear screen render pass"),
                occlusion_query_set: None,
            });
        }

        // --- Draw vertical dashed green line at center ---
        if state.pause_menu.is_debug_panel_visible() {
            let w = state.surface_config.width as f32;
            let h = state.surface_config.height as f32;
            let center_x = w / 2.0;
            let dash_height: f32 = 16.0;
            let dash_gap: f32 = 12.0;
            let dash_width = 3.0;
            let color = [0.1, 1.0, 0.1, 0.85]; // bright green, mostly opaque
            let mut dashes = Vec::new();
            let mut y = 0.0;
            while y < h {
                let dash_h = dash_height.min(h - y);
                dashes.push(crate::rectangle::Rectangle::new(
                    center_x - dash_width / 2.0,
                    y,
                    dash_width,
                    dash_h,
                    color,
                ));
                y += dash_height + dash_gap;
            }
            // Use the pause_menu's rectangle_renderer for simplicity (always present)
            let renderer = &mut state.pause_menu.button_manager.rectangle_renderer;
            for dash in dashes {
                renderer.add_rectangle(dash);
            }
            // Render the dashes before anything else
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                label: Some("center line render pass"),
                occlusion_query_set: None,
            });
            renderer.render(&state.device, &mut render_pass);
        }
        // --- End vertical dashed line ---

        // --- Game UI: update and render timer/score/level ---

        // --- Debug Info Panel ---
        if state.pause_menu.is_debug_panel_visible() {
            let window_size = &state.surface_config;
            let debug_text = format!(
                "Window Size: {} x {}",
                window_size.width, window_size.height
            );
            use crate::text::{TextPosition, TextStyle};
            use glyphon::Color;
            let style = TextStyle {
                font_family: "HankenGrotesk".to_string(),
                font_size: 22.0,
                line_height: 26.0,
                color: Color::rgb(220, 40, 40),
                weight: glyphon::Weight::BOLD,
                style: glyphon::Style::Normal,
            };
            let pos = TextPosition {
                x: window_size.width as f32 - 320.0,
                y: 20.0,
                max_width: Some(300.0),
                max_height: Some(40.0),
            };
            state.text_renderer.create_text_buffer(
                "debug_info",
                &debug_text,
                Some(style),
                Some(pos),
            );
        } else {
            // Hide debug info by making it transparent if it exists
            if let Some(buf) = state.text_renderer.text_buffers.get_mut("debug_info") {
                buf.visible = false;
            }
        }
        // Prepare and render text BEFORE pause menu overlay
        if let Err(e) =
            state
                .text_renderer
                .prepare(&state.device, &state.queue, &state.surface_config)
        {
            println!("Failed to prepare text renderer: {}", e);
        }
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                label: Some("text render pass"),
                occlusion_query_set: None,
            });
            if let Err(e) = state.text_renderer.render(&mut render_pass) {
                println!("Failed to render text: {}", e);
            }
        }
        // --- End Game UI ---

        // Render pause menu if visible (overlay comes after text)
        if state.pause_menu.is_visible() {
            // Prepare pause menu for rendering
            if let Err(e) =
                state
                    .pause_menu
                    .prepare(&state.device, &state.queue, &state.surface_config)
            {
                println!("Failed to prepare pause menu: {}", e);
            }

            // Create a render pass for the pause menu
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                label: Some("pause menu render pass"),
                occlusion_query_set: None,
            });

            // --- Add semi-transparent grey overlay ---
            let overlay_color = [0.08, 0.09, 0.11, 0.88]; // darker, neutral semi-transparent grey
            let (w, h) = (
                state.surface_config.width as f32,
                state.surface_config.height as f32,
            );
            state
                .pause_menu
                .button_manager
                .rectangle_renderer
                .add_rectangle(crate::rectangle::Rectangle::new(
                    0.0,
                    0.0,
                    w,
                    h,
                    overlay_color,
                ));
            state
                .pause_menu
                .button_manager
                .rectangle_renderer
                .render(&state.device, &mut render_pass);
            // --- End overlay ---

            // Render the pause menu
            if let Err(e) = state.pause_menu.render(&state.device, &mut render_pass) {
                println!("Failed to render pause menu: {}", e);
            }
            state.pause_menu_state = PauseMenuState::Open;
        } else {
            // Explicitly clear rectangles if menu is not visible
            state
                .pause_menu
                .button_manager
                .rectangle_renderer
                .clear_rectangles();
            state.pause_menu_state = PauseMenuState::Closed;
        }

        state.queue.submit(Some(encoder.finish()));
        surface_texture.present();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        pollster::block_on(self.set_window(window));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();

        // Handle pause menu input first
        state.pause_menu.handle_input(&event);

        // Check for pause menu actions
        match state.pause_menu.get_last_action() {
            PauseMenuAction::Resume => {
                state.pause_menu.hide();
            }
            PauseMenuAction::Settings => {
                // TODO: Implement settings menu
            }
            PauseMenuAction::Restart => {
                // TODO: Implement level restart
            }
            PauseMenuAction::QuitToMenu => {
                event_loop.exit();
            }

            PauseMenuAction::None => {}
        }

        // Handle keyboard events for pause menu toggle
        if let WindowEvent::KeyboardInput { event, .. } = &event {
            if event.state == ElementState::Pressed {
                if let winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) =
                    event.physical_key
                {
                    state.pause_menu.toggle();
                    if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                    }
                }
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.handle_redraw();
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(new_size) => {
                self.handle_resized(new_size.width, new_size.height);
            }
            _ => (),
        }
    }
}
