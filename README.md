# Pause Menu Project (Rust)

## Overview
This project is a Rust application (likely a game or interactive app) featuring a custom UI system for rendering menus and buttons. It uses GPU-accelerated rendering with `wgpu` and `glyphon`, and is structured for modularity and flexibility.

---

## Project Structure
- **src/pause_menu.rs**: Implements the Pause Menu overlay, including button creation, layout, input handling, and rendering.
- **src/button.rs**: Defines the button system, including `Button`, `ButtonManager`, and style presets.
- **src/text.rs**: Handles text rendering and measurement.
- **src/rectangle.rs**: Likely handles drawing rectangles for UI backgrounds.
- **src/main.rs, src/app.rs, src/game.rs**: Main application/game logic and entry point.

---

## Button Sizing & Layout
- **Width**: Fixed at 420.0 pixels for main menu buttons.
- **Height**: Proportional to window height (e.g., 13% or 20%).
- **Spacing**: 8.0 pixels between buttons.
- **Text Sizing**: Font size and line height scale with window height for responsiveness.
- **Vertical Centering**: Buttons are centered as a group using calculated positions.

---

## Button Styling & Presets
Button styles are created using preset functions in `button.rs`:
- `create_primary_button_style` (main/default)
- `create_goldenrod_button_style` (yellow/gold)
- `create_lobby_button_style` (less saturated red)
- `create_danger_button_style` (red/danger)
- `create_warning_button_style` (orange/warning)

Each preset defines background color, text style, padding, and other visual properties. Text style includes font family, size, color, weight, and style.

---

## ButtonSpacing: Wrap vs Hbar(f32)
- **Wrap**: Text wraps onto multiple lines to fit the button width. Used for small or square buttons, or long labels.
- **Hbar(f32)**: Single-line layout with a specified amount of horizontal spacing. Used for standard/wide buttons.

Example enum:
```rust
pub enum ButtonSpacing {
    Wrap,
    Hbar(f32),
}
```

---

## How to Add or Expand Styles
1. Define a new style function in `button.rs` (clone and modify an existing one).
2. Use the new style when creating a button in `pause_menu.rs` or elsewhere.
3. Adjust sizing/spacing as needed for custom layouts.

---

## Rendering & Input
- Uses `wgpu` for GPU rendering and `glyphon` for text.
- ButtonManager handles button state, input, and rendering.
- PauseMenu manages visibility, resizing, and user actions.

---

## Summary Table: Style Presets
| Preset Function                  | Used For         | Likely Color/Style      |
|----------------------------------|------------------|------------------------|
| create_primary_button_style      | Resume           | Main/Default           |
| create_goldenrod_button_style    | Restart Run      | Yellow/Gold            |
| create_lobby_button_style        | Quit to Lobby    | Less saturated red     |
| create_danger_button_style       | Quit App         | Red/Danger             |
| create_warning_button_style      | Debug Info       | Orange/Warning         |

---

## Extending the System
- Add new style presets in `button.rs`.
- Add new buttons in `pause_menu.rs` using your custom styles.
- Adjust layout logic for new button arrangements or behaviors.

---

For more details, see the source files in the `src/` directory. 

---

## Example: Creating a Centered Resume Button

Here’s an example of how a styled, centered "Resume Game" button is created in the pause menu:

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

### Explanation

1. **Create a Style Preset**
   - Calls a function (`create_primary_button_style`) that returns a default style for primary buttons (background color, border, etc.).
   - The style is mutable so it can be customized further.

2. **Customize the Text Style**
   - Overrides the default text style with a dynamically scaled `text_style` (likely calculated based on the window size for responsive design).

3. **Create the Button**
   - Instantiates a new `Button` with the ID `"resume"` and the label `"Resume Game"`.

4. **Apply the Style**
   - Applies the customized style to the button.

5. **Set Text Alignment**
   - Ensures the button label is centered within the button.

6. **Set Position and Size**
   - Positions the button horizontally at `center_x` and vertically at the position calculated by `y(0)` (the first button in the menu).
   - Sets the button’s width and height.
   - Anchors the button’s position to its center, so it’s truly centered on the screen.

**Summary:**
This snippet demonstrates how to use a style preset, customize it for responsive text, create a button with a unique ID and label, center the button both visually and textually, and precisely control the button’s size and position in the menu. 