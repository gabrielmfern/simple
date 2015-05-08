
use std;

extern crate sdl2;
extern crate sdl2_image;
use sdl2::render;
use sdl2::video;
use sdl2::pixels;
use sdl2_image::LoadTexture;

use std::path::Path;

use event::{self,Event};
use shape;

/**
 * A Window can display graphics and handle events.
 *
 * A Window has a draw color at all times, and that color is applied to every operation. If you set
 * the color to `(255, 0, 0)`, all drawn graphics and images will have a red tint.
 *
 * Creating multiple Windows is untested and will probably crash!
 *
 */
pub struct Window<'a> {
    // sdl graphics
    context:                    sdl2::sdl::Sdl,
    renderer:                   render::Renderer<'a>,
    foreground_color:           pixels::Color,

    // events and event logic
    running:                    bool,
    event_queue:                std::vec::Vec<Event>,

    // timing
    target_ticks_per_frame:     u32,
    ticks_at_previous_frame:    u32,
}

/// Top-level Running / Creation Methods
/// ====================================
impl<'a> Window<'a> {
    /// Intialize a new running window. `name` is used as a caption.
    pub fn new(name: &str, width: u16, height: u16) -> Self {
        // SDL2 Initialization calls. This section here is the reason we can't easily create
        // multiple Windows. There would have to be some kind of global variable that tracked
        // whether SDL2 had already been init'd.
        //
        // Note that initialization is not the only problem. SDL2 is usually safe to init
        // multiple times, but it's not safe to de-init SDL2 and then continue using it. We'd
        // either have to have an explicit Deinitialize() global function or keep a global count
        // of windows that exist.
        //
        // Both solutions are ugly and error-prone, and would probably break thread safety. Going
        // to assume that there will only be one Window per program.
        //
        // TODO: solve this problem
        //
        let sdl_context = sdl2::init(sdl2::INIT_EVERYTHING).unwrap();
        sdl2_image::init(sdl2_image::InitFlag::all());
        let sdl_window = video::Window::new(
            &sdl_context,
            name,
            video::WindowPos::PosUndefined,
            video::WindowPos::PosUndefined,
            width as i32, height as i32,
            video::SHOWN,
        ).unwrap();

        let mut renderer = render::Renderer::from_window(
            sdl_window,
            render::RenderDriverIndex::Auto,
            render::ACCELERATED,
        ).unwrap();

        // for transparency
        renderer.drawer().set_blend_mode(render::BlendMode::Blend);

        let mut window = Window{
            context:                    sdl_context,
            renderer:                   renderer,
            running:                    true,
            event_queue:                vec![],
            foreground_color:           pixels::Color::RGBA(0, 0, 0, 255),
            target_ticks_per_frame:     (1000.0 / 60.0) as u32,
            ticks_at_previous_frame:    0,
        };
        window.clear();
        window
    }

    /// Redrawing and update the display, while maintaining a consistent framerate and updating the
    /// event queue. You should draw your objects immediately before you call this function.
    ///
    /// NOTE: This function returns false if the program should terminate. This allows for nice
    /// constructs like `while app.next_frame() { ... }`
    pub fn next_frame(&mut self) -> bool {
        if !self.running {
            return false;
        }

        self.renderer.drawer().present();

        let mut current_ticks = sdl2::timer::get_ticks();
        while current_ticks - self.ticks_at_previous_frame < self.target_ticks_per_frame {
            sdl2::timer::delay(5);
            current_ticks = sdl2::timer::get_ticks();
        }
        self.ticks_at_previous_frame = current_ticks;

        // Handle events
        loop {
            let sdl_event = self.context.event_pump().poll_event();
            match sdl_event {
                None => break,
                Some(sdl_event) => match Event::from_sdl2_event(sdl_event) {
                    Some(Event::Quit) => self.quit(),

                    // any other unrecognized event
                    Some(e) => (self.event_queue.push(e)),
                    None => (),
                },
            };
        }

        true
    }

    /// Return true when there is an event waiting in the queue for processing.
    pub fn has_event(&self) -> bool { self.event_queue.len() > 0 }

    /// Get the next event from the queue. NOTE: If the event queue on the Window is empty, this
    /// function will panic. Call `has_event()` to find out if there is an event ready for
    /// processing.
    ///
    /// Note that events are handled in a first-in-first-out order. If a user presses three keys 1,
    /// 2, 3 during a frame, then the next three calls to next_event will return 1, 2, 3 in the
    /// same order.
    pub fn next_event(&mut self) -> Event { self.event_queue.remove(0) }

    /// Return true if the button is currently pressed. NOTE: This function is probably not
    /// performant.
    pub fn is_key_down(&self, key: event::Key) -> bool {
        // TODO: this has got to be slow but I can't figure out a way to get the state of
        // individual keys from sdl2-rs.
        let state = sdl2::keyboard::get_keyboard_state();
        match state.get(&key) {
            Some(ref b) if **b => true,
            _ => false,
        }
    }

    /// Return true if the specified button is down. NOTE: Unknown mouse buttons are NOT handled
    /// and will always return `false`.
    pub fn is_mouse_button_down(&self, button: event::MouseButton) -> bool {
        let flags = sdl2::mouse::get_mouse_state().0;
        match event::mousebutton_to_mousestate(button) {
            Some(state) => flags.contains(state),
            None => false,
        }
    }

    // Return the current position of the mouse, relative to the top-left corner of the Window.
    pub fn mouse_position(&self) -> (i32, i32) {
        let state = sdl2::mouse::get_mouse_state();
        (state.1, state.2)
    }

    /// This does not actually cause the program to exit. It just means that next_frame will return
    /// false on the next call.
    pub fn quit(&mut self) {
        self.running = false;
    }
}

/// Drawing Methods
/// ===============
impl<'a> Window<'a> {
    /// Windows have a color set on them at all times. This color is applied to every draw
    /// operation. To "unset" the color, call set_color with (255,255,255,255)
    pub fn set_color(&mut self, red: u8, green: u8, blue: u8, alpha: u8) {
        self.foreground_color = pixels::Color::RGBA(red, green, blue, alpha);
    }

    /// Set up the color according to the internal state of the Window.
    fn prepare_to_draw(&mut self) {
        self.renderer.drawer().set_draw_color(self.foreground_color);
    }

    // These functions are just aliases onto self.renderer.drawer() as you can see.
    pub fn draw_rect(&mut self, rect: shape::Rect)     {
        self.prepare_to_draw();
        self.renderer.drawer().draw_rect(rect)
    }
    pub fn fill_rect(&mut self, rect: shape::Rect)     {
        self.prepare_to_draw();
        self.renderer.drawer().fill_rect(rect)
    }
    pub fn draw_point(&mut self, point: shape::Point)  {
        self.prepare_to_draw();
        self.renderer.drawer().draw_point(point)
    }
    pub fn draw_polygon(&mut self, polygon: shape::Polygon) {
        self.prepare_to_draw();
        self.renderer.drawer().draw_points(&polygon[..])
    }

    /// Display the image with its top-left corner at (x, y)
    pub fn draw_image(&mut self, image: &mut Image, x: i32, y: i32) {
        // first, configure the texture for drawing according to the current foreground_color
        let (r,g,b,a) = match self.foreground_color {
            pixels::Color::RGB(r, g, b) => (r,g,b,255),
            pixels::Color::RGBA(r, g, b, a) => (r,g,b,a),
        };
        image.texture.set_color_mod(r, g, b);
        image.texture.set_alpha_mod(a);

        // copy the texture onto the drawer()
        self.renderer.drawer().copy(&(image.texture), Some(shape::Rect{
            x: x,
            y: y,
            w: image.get_width(),
            h: image.get_height(),
        }), None);
    }

    /// Clear the screen to black. This will set the Window's draw color to (0,0,0,255)
    pub fn clear(&mut self) {
        self.set_color(0, 0, 0, 255);
        self.prepare_to_draw();
        self.renderer.drawer().clear();
    }
}

/**
 * Image represents a texture that can be drawn on the screen.
 *
 * Images are immutable, in the sense that they have no methods to modify their contents.
 */
pub struct Image {
    texture:    render::Texture,
    width:      i32,
    height:     i32,
}

impl Image {
    pub fn get_width(&self) -> i32  { self.width }
    pub fn get_height(&self) -> i32 { self.height }
}

/// Resource Loading Methods
/// ========================
impl<'a> Window<'a> {
    /// Load the image at the path you specify.
    pub fn load_image(&self, filename: &Path) -> Result<Image,String> {
        let mut texture = try!(LoadTexture::load_texture(&(self.renderer), &filename));
        texture.set_blend_mode(render::BlendMode::Blend);
        Ok(Image{
            width:      texture.query().width,
            height:     texture.query().height,
            texture:    texture,
        })
    }

    // TODO: font loading support
    //
    // https://github.com/alexandercampbell/simple/issues/10
}

// Dtor for Window.
impl<'a> std::ops::Drop for Window<'a> {
    /// Close the window and clean up resources.
    fn drop(&mut self) {
        sdl2_image::quit();
    }
}

