# Simplified Game UI Development Environment

*A streamlined version of the [Mirador game engine](https://github.com/DetectiveFierce/mirador) focused on UI development and testing*

---

## Introduction

This project is a simplified version of the game engine used in [Mirador](https://github.com/DetectiveFierce/mirador), designed specifically as a development environment for testing and refining button systems, menu layouts, and UI interactions. It provides a focused environment for developing UI components without the complexity of the full game engine.

The application uses the same core technologies as Mirador - WGPU for GPU-accelerated rendering, Glyphon for text rendering, and a custom state management system - but stripped down to focus on UI development needs.

---

## Purpose

This development environment serves several key purposes:

- **UI Component Testing**: Rapid iteration on button designs, layouts, and interactions
- **Menu System Development**: Testing pause menus, navigation flows, and state transitions
- **Responsive Design Validation**: Ensuring UI elements scale properly across different window sizes
- **Input Handling Verification**: Testing keyboard and mouse interactions in isolation
- **Rendering Pipeline Testing**: Validating GPU-accelerated UI rendering performance

The simplified architecture allows developers to focus on UI concerns without the overhead of game logic, physics, audio, or other complex systems present in the full Mirador engine.

---

## Architecture Overview

### Core Components

The development environment maintains the same modular structure as the full engine:

- **`app.rs`**: Main application loop and event handling
- **`game.rs`**: Simplified game state management and UI logic
- **`pause_menu.rs`**: Pause menu overlay with button management
- **`button.rs`**: Reusable button system with style presets
- **`text.rs`**: Text rendering and measurement utilities
- **`rectangle.rs`**: GPU-accelerated rectangle rendering

### State Management

The application uses a simplified state management system centered around the `CurrentScreen` enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentScreen {
    Loading,
    Game,
    Pause,
    GameOver,
    NewGame,
}
```

This provides a clean foundation for testing UI state transitions without the complexity of full game state management.

---

## Recent Architecture Improvements

### Unified State Management

The development environment consolidates UI state management into a single `CurrentScreen` enum, eliminating the need for separate pause menu state tracking:

```rust
// Single source of truth for all screen states
match current_screen {
    CurrentScreen::Game => game_ui.resume_timer(),
    CurrentScreen::Pause => game_ui.pause_timer(),
    _ => game_ui.pause_timer(),
}
```

### Timer Integration

The countdown timer serves as a practical test case for UI state management:

- **Active Timer**: Only counts down when `current_screen == Game`
- **Paused Timer**: Automatically pauses when entering pause menu
- **State Transitions**: Timer state changes are handled during screen transitions

```rust
// Timer control during state transitions
match state.pause_menu.get_last_action() {
    PauseMenuAction::Resume => {
        state.game_state.current_screen = CurrentScreen::Game;
        state.game_state.game_ui.resume_timer();
    }
    // ... other actions
}
```

### Continuous Rendering

The development environment implements continuous rendering to ensure smooth UI updates:

```rust
// Request another redraw to keep the timer updating
if let Some(window) = self.window.as_ref() {
    window.request_redraw();
}
```

This provides a realistic testing environment for UI responsiveness and performance.

---

## Rendering Pipeline

### Multi-Pass Rendering

The development environment uses the same multi-pass rendering approach as the full engine:

1. **Background Clear**: Consistent visual foundation
2. **Debug Overlay**: Optional center line for development/debugging
3. **Game UI**: Timer, score, and level information
4. **Pause Menu**: Semi-transparent overlay with interactive buttons

### GPU-Accelerated Rendering

All rendering is handled through WGPU for optimal performance:

```rust
let mut encoder = state.device.create_command_encoder(
    &wgpu::CommandEncoderDescriptor { label: None }
);

// Multiple render passes for different UI layers
let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: &surface_view,
        resolve_target: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.18, g: 0.24, b: 0.32, a: 1.0 }),
            store: wgpu::StoreOp::Store,
        },
    })],
    // ... other configuration
});
```

### Text Rendering with Glyphon

High-quality text rendering is achieved through Glyphon:

```rust
let style = TextStyle {
    font_family: "HankenGrotesk".to_string(),
    font_size: 22.0,
    line_height: 26.0,
    color: Color::rgb(220, 40, 40),
    weight: glyphon::Weight::BOLD,
    style: glyphon::Style::Normal,
};
```

---

## Button System Architecture

### Responsive Design

The button system implements true responsive design with dynamic sizing:

```rust
// Button dimensions scale with window size
let button_width = 420.0; // Fixed width for consistency
let button_height = window_height * 0.13; // Proportional height
let spacing = 8.0; // Consistent spacing
```

### Style Presets

The development environment includes a comprehensive set of button style presets for testing different visual styles:

| Preset Function | Purpose | Visual Style |
|----------------|---------|--------------|
| `create_primary_button_style` | Resume Game | Main/Default blue |
| `create_goldenrod_button_style` | Restart Run | Yellow/Gold accent |
| `create_lobby_button_style` | Quit to Lobby | Muted red |
| `create_danger_button_style` | Quit App | Bright red warning |
| `create_warning_button_style` | Debug Info | Orange warning |

### Button Layout System

Buttons support flexible layout options for testing different UI patterns:

```rust
pub enum ButtonSpacing {
    Wrap,           // Text wraps to multiple lines
    Hbar(f32),      // Single line with horizontal spacing
}
```

### Example: Creating a Centered Resume Button

```rust
let mut resume_style = create_primary_button_style();
resume_style.text_style = text_style.clone();
let resume_button = Button::new("resume", "Resume Game")
    .with_style(resume_style)
    .with_text_align(TextAlign::Center)
    .with_position(
        ButtonPosition::new(center_x, y(0), button_width, button_height)
            .with_anchor(ButtonAnchor::Center),
    );
```

This demonstrates:
- Style preset usage with customization
- Responsive text scaling
- Precise positioning with center anchoring
- Consistent sizing and spacing

---

## Event Handling and Input

### Keyboard Input

The development environment handles keyboard events for testing pause menu toggling:

```rust
if let winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) = event.physical_key {
    if state.game_state.current_screen == CurrentScreen::Pause {
        state.game_state.current_screen = CurrentScreen::Game;
        state.game_state.game_ui.resume_timer();
    } else {
        state.game_state.current_screen = CurrentScreen::Pause;
        state.game_state.game_ui.pause_timer();
    }
}
```

### Mouse Input

Button interactions are handled through the `ButtonManager` for testing UI responsiveness:

```rust
// Handle pause menu input when visible
if state.game_state.current_screen == CurrentScreen::Pause && state.pause_menu.is_visible() {
    state.pause_menu.handle_input(&event);
    // Process menu actions
}
```

### Window Events

The development environment properly handles window resizing and lifecycle events:

```rust
match event {
    WindowEvent::CloseRequested => event_loop.exit(),
    WindowEvent::RedrawRequested => self.handle_redraw(),
    WindowEvent::Resized(new_size) => self.handle_resized(new_size.width, new_size.height),
    _ => (),
}
```

---

## Performance Considerations

### Efficient Rendering

- **Conditional Rendering**: UI elements are only rendered when needed
- **GPU Acceleration**: All rendering operations use WGPU for optimal performance
- **Minimal State Updates**: Timer and UI updates are batched efficiently

### Memory Management

- **Arc<Window>**: Shared window reference for efficient event handling
- **Surface Management**: Proper surface configuration and cleanup
- **Texture Reuse**: Efficient texture view creation and management

---

## Extending the System

### Adding New Button Styles

1. Define a new style function in `button.rs`:
```rust
pub fn create_custom_button_style() -> ButtonStyle {
    ButtonStyle {
        background_color: [0.2, 0.8, 0.4, 1.0], // Custom green
        text_style: TextStyle { /* custom text style */ },
        // ... other properties
    }
}
```

2. Use the new style in your UI:
```rust
let custom_button = Button::new("custom", "Custom Action")
    .with_style(create_custom_button_style())
    .with_position(/* position */);
```

### Adding New Game States

1. Extend the `CurrentScreen` enum:
```rust
pub enum CurrentScreen {
    Loading,
    Game,
    Pause,
    GameOver,
    NewGame,
    Settings, // New state
}
```

2. Handle the new state in your rendering and input logic:
```rust
match current_screen {
    CurrentScreen::Settings => {
        // Render settings UI
        // Handle settings input
    }
    // ... other states
}
```

### Custom Timer Configurations

The timer system supports custom configurations for testing different timing scenarios:

```rust
let config = TimerConfig {
    duration: Duration::from_secs(60),
    warning_threshold: Duration::from_secs(10),
    // ... other options
};
game_ui.start_timer(Some(config));
```

---

## Conclusion

This development environment provides a focused platform for UI development and testing, using the same core technologies as the full Mirador game engine but simplified for rapid iteration on UI components. The architecture supports:

- **Rapid Prototyping**: Quick testing of button designs and layouts
- **State Management Testing**: Validation of UI state transitions
- **Performance Validation**: Testing rendering performance and responsiveness
- **Cross-Platform Compatibility**: Same rendering pipeline as the full engine

The simplified structure allows developers to focus on UI concerns while maintaining compatibility with the full engine's architecture and rendering pipeline.

---

## Getting Started

To run the development environment:

```bash
cargo run
```

The application will start in Game mode with a countdown timer. Press `Escape` to toggle the pause menu, and use the mouse to interact with menu buttons.

For development and debugging, the application includes optional debug overlays that can be enabled through the pause menu. 