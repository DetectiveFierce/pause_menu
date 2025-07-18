use crate::ui::button::{
    create_primary_button_style, Button, ButtonAnchor, ButtonManager, ButtonPosition, TextAlign,
};
use egui_wgpu::wgpu::{self, Device, Queue, RenderPass, SurfaceConfiguration};
use glyphon::{Color, Resolution};
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::Window;

#[derive(Debug, Clone, PartialEq)]
pub enum UpgradeMenuAction {
    SelectUpgrade1,
    SelectUpgrade2,
    SelectUpgrade3,
    None,
}

pub struct UpgradeMenu {
    pub button_manager: ButtonManager,
    pub visible: bool,
    pub last_action: UpgradeMenuAction,
}

impl UpgradeMenu {
    pub fn new(
        device: &Device,
        queue: &Queue,
        surface_format: wgpu::TextureFormat,
        window: &Window,
    ) -> Self {
        let mut button_manager = ButtonManager::new(device, queue, surface_format, window);

        // Create upgrade menu layout
        Self::create_upgrade_layout(&mut button_manager, window.inner_size());

        Self {
            button_manager,
            visible: false,
            last_action: UpgradeMenuAction::None,
        }
    }

    fn create_upgrade_layout(button_manager: &mut ButtonManager, window_size: PhysicalSize<u32>) {
        let window_width = window_size.width as f32;
        let window_height = window_size.height as f32;

        // Main container dimensions (large rounded rectangle)
        let container_width = window_width * 0.8;
        let container_height = window_height * 0.7;
        let container_x = (window_width - container_width) / 2.0;
        let container_y = (window_height - container_height) / 2.0;

        // Store container dimensions for rendering
        button_manager.container_rect = Some(
            crate::ui::rectangle::Rectangle::new(
                container_x,
                container_y,
                container_width,
                container_height,
                [0.4, 0.4, 0.4, 1.0], // Medium grey
            )
            .with_corner_radius(20.0),
        );

        // Three upgrade slots (tall rounded rectangles)
        let slot_width = container_width * 0.25; // 25% of container width
        let slot_spacing = container_width * 0.05; // 5% spacing between slots
        let total_slots_width = slot_width * 3.0 + slot_spacing * 2.0;
        let slots_start_x = container_x + (container_width - total_slots_width) / 2.0;

        // Create three upgrade slot buttons
        for i in 0..3 {
            let slot_x = slots_start_x + i as f32 * (slot_width + slot_spacing);

            // Create a custom style for the upgrade slots (lighter grey)
            let mut slot_style = create_primary_button_style();
            slot_style.background_color = Color::rgb(200, 200, 200); // Light grey
            slot_style.hover_color = Color::rgb(180, 180, 180); // Slightly darker on hover
            slot_style.pressed_color = Color::rgb(160, 160, 160); // Even darker when pressed
            slot_style.corner_radius = 12.0; // Rounded corners
            slot_style.padding = (8.0, 8.0); // Minimal padding
            slot_style.text_style.font_size = 32.0; // Doubled from 16.0
            slot_style.text_style.line_height = 48.0; // Doubled from 18.0 (approximate)
            slot_style.text_style.color = Color::rgb(50, 50, 50); // Dark text for contrast

            let upgrade_text = match i {
                0 => "Upgrade 1",
                1 => "Upgrade 2",
                2 => "Upgrade 3",
                _ => "Unknown",
            };

            // Calculate height proportion for tall buttons
            let margin = 0.1; // 10% margin
            let height_proportion = (container_height * (1.0 - 2.0 * margin)) / window_height;
            slot_style.spacing = crate::ui::button::ButtonSpacing::Tall(height_proportion);

            let button = Button::new(&format!("upgrade_{}", i + 1), upgrade_text)
                .with_style(slot_style)
                .with_text_align(TextAlign::Center)
                .with_level_text()
                .with_tooltip_text()
                .with_position(
                    ButtonPosition::new(slot_x, 0.0, slot_width, 0.0) // Width set, height will be calculated by ButtonManager
                        .with_anchor(ButtonAnchor::TopLeft),
                );

            button_manager.add_button(button);
        }

        // Update button positions to ensure proper layout
        button_manager.update_button_positions();
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.last_action = UpgradeMenuAction::None;

        // Show all buttons
        for button in self.button_manager.buttons.values_mut() {
            button.set_visible(true);
        }

        // Ensure button text is made visible and styled immediately
        self.button_manager.update_button_states();
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.last_action = UpgradeMenuAction::None;

        // Hide all buttons
        for button in self.button_manager.buttons.values_mut() {
            button.set_visible(false);
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn handle_input(&mut self, event: &WindowEvent) {
        if !self.visible {
            return;
        }

        self.button_manager.handle_input(event);

        // Check for button clicks
        if self.button_manager.is_button_clicked("upgrade_1") {
            self.last_action = UpgradeMenuAction::SelectUpgrade1;
        }

        if self.button_manager.is_button_clicked("upgrade_2") {
            self.last_action = UpgradeMenuAction::SelectUpgrade2;
        }

        if self.button_manager.is_button_clicked("upgrade_3") {
            self.last_action = UpgradeMenuAction::SelectUpgrade3;
        }
    }

    pub fn get_last_action(&mut self) -> UpgradeMenuAction {
        let action = self.last_action.clone();
        self.last_action = UpgradeMenuAction::None;
        action
    }

    pub fn resize(&mut self, queue: &Queue, resolution: Resolution) {
        self.button_manager.resize(queue, resolution);
        // Update window_size for correct centering
        self.button_manager.window_size = winit::dpi::PhysicalSize {
            width: resolution.width,
            height: resolution.height,
        };
        // Recreate layout with new positions if menu is visible
        if self.visible {
            self.recreate_layout_for_new_size();
        }
    }

    fn recreate_layout_for_new_size(&mut self) {
        let window_size = self.button_manager.window_size;
        let window_width = window_size.width as f32;
        let window_height = window_size.height as f32;

        // Main container dimensions
        let container_width = window_width * 0.8;
        let container_height = window_height * 0.7;
        let container_x = (window_width - container_width) / 2.0;
        let container_y = (window_height - container_height) / 2.0;

        // Update container rectangle
        self.button_manager.container_rect = Some(
            crate::ui::rectangle::Rectangle::new(
                container_x,
                container_y,
                container_width,
                container_height,
                [0.4, 0.4, 0.4, 1.0], // Medium grey
            )
            .with_corner_radius(20.0),
        );

        // Three upgrade slots
        let slot_width = container_width * 0.25;
        let slot_spacing = container_width * 0.05;
        let total_slots_width = slot_width * 3.0 + slot_spacing * 2.0;
        let slots_start_x = container_x + (container_width - total_slots_width) / 2.0;

        // Update positions for all three upgrade buttons
        for i in 0..3 {
            let slot_x = slots_start_x + i as f32 * (slot_width + slot_spacing);

            // Calculate height proportion for tall buttons
            let margin = 0.1; // 10% margin
            let height_proportion = (container_height * (1.0 - 2.0 * margin)) / window_height;

            if let Some(button) = self
                .button_manager
                .get_button_mut(&format!("upgrade_{}", i + 1))
            {
                // Update the spacing to use the new height proportion
                button.style.spacing = crate::ui::button::ButtonSpacing::Tall(height_proportion);
                button.position.x = slot_x;
                button.position.y =
                    container_y + (container_height - (window_height * height_proportion)) / 2.0; // Center vertically
                button.position.width = slot_width;
                button.position.anchor = ButtonAnchor::TopLeft;
            }
        }

        // Update text positions
        self.button_manager.update_button_positions();
    }

    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        surface_config: &SurfaceConfiguration,
    ) -> Result<(), glyphon::PrepareError> {
        self.button_manager.prepare(device, queue, surface_config)
    }

    pub fn render(
        &mut self,
        device: &Device,
        render_pass: &mut RenderPass,
    ) -> Result<(), glyphon::RenderError> {
        self.button_manager.render(device, render_pass)
    }
}
