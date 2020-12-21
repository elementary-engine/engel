use std::{
    borrow::Cow,
    error::Error,
    path::Path,
    thread,
    time::{Duration, Instant},
};

use engel_core::{controller, Color, Comp, KeyboardController, MouseController, Real, Render, SystemMessage};
pub use gl;
pub use glutin::{
    self,
    dpi::Pixel,
    event_loop::ControlFlow,
    monitor::{MonitorHandle, VideoMode},
    window::{BadIcon, Fullscreen, Icon, WindowBuilder},
    Api, Context, ContextBuilder, ContextError, CreationError, GlProfile, GlRequest, NotCurrent, Robustness, GL_CORE,
};
use glutin::{
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
    PossiblyCurrent, WindowedContext,
};
use thiserror::Error;

#[derive(Copy, Clone, Debug)]
pub enum Buffering {
    Single,
    Double,
    DoNotCare,
}

impl From<Buffering> for Option<bool> {
    fn from(buffering: Buffering) -> Self {
        match buffering {
            Buffering::Single => Some(false),
            Buffering::Double => Some(true),
            Buffering::DoNotCare => None,
        }
    }
}

impl From<Option<bool>> for Buffering {
    fn from(buffering: Option<bool>) -> Self {
        match buffering {
            Some(false) => Buffering::Single,
            Some(true) => Buffering::Double,
            None => Buffering::DoNotCare,
        }
    }
}

impl From<bool> for Buffering {
    fn from(double_buffering: bool) -> Self {
        Self::from(Some(double_buffering))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Acceleration {
    Hardware,
    Software,
    DoNotCare,
}

impl From<Acceleration> for Option<bool> {
    fn from(acceleration: Acceleration) -> Self {
        match acceleration {
            Acceleration::Software => Some(false),
            Acceleration::Hardware => Some(true),
            Acceleration::DoNotCare => None,
        }
    }
}

impl From<Option<bool>> for Acceleration {
    fn from(acceleration: Option<bool>) -> Self {
        match acceleration {
            Some(false) => Acceleration::Software,
            Some(true) => Acceleration::Hardware,
            None => Acceleration::DoNotCare,
        }
    }
}

impl From<bool> for Acceleration {
    fn from(acceleration: bool) -> Self {
        Self::from(Some(acceleration))
    }
}

#[derive(Debug, Error)]
pub enum AppError<RE: Error> {
    #[error("Create application window error: {:?}", .0)]
    CreationError(#[from] CreationError),

    #[error("Context manipulation error: {:?}", .0)]
    ContextError(#[from] ContextError),

    #[error("Renderer internal error: {:?}", .0)]
    RendererError(RE),
}

pub enum AppState {
    Exit,
    Continue,
}

struct Font<'a> {
    name: Cow<'a, str>,
    path: Cow<'a, Path>,
}

pub struct App<'a, R> {
    window_builder: WindowBuilder,
    context_builder: ContextBuilder<'a, NotCurrent>,
    renderer: R,
    background_color: Color,
    exit_by_escape: bool,
    font: Option<Font<'a>>,
}

impl<'a, R: Render + 'static> App<'a, R> {
    #[inline]
    pub fn new(renderer: R) -> Self {
        App {
            window_builder: WindowBuilder::new(),
            context_builder: ContextBuilder::new(),
            renderer,
            background_color: Color::RGBA(0.8, 0.8, 0.8, 1.0),
            exit_by_escape: true,
            font: None,
        }
    }

    /// Requests the window to be of specific dimensions.
    ///
    /// See [`glutin::window::Window::set_inner_size`] for details.
    #[inline]
    pub fn with_inner_size<P: Pixel>(mut self, width: P, height: P) -> Self {
        self.window_builder.window.inner_size = Some(PhysicalSize::new(width, height).into());
        self
    }

    /// Requests the window to be of specific dimensions.
    ///
    /// See [`glutin::window::Window::set_inner_size`] for details.
    #[inline]
    pub fn with_logical_inner_size<P: Pixel>(mut self, width: P, height: P) -> Self {
        self.window_builder.window.inner_size = Some(LogicalSize::new(width, height).into());
        self
    }

    /// Sets a minimum dimension size for the window.
    ///
    /// See [`glutin::window::Window::set_min_inner_size`] for details.
    #[inline]
    pub fn with_min_inner_size<P: Pixel>(mut self, width: P, height: P) -> Self {
        self.window_builder.window.min_inner_size = Some(PhysicalSize::new(width, height).into());
        self
    }

    /// Sets a minimum dimension size for the window.
    ///
    /// See [`glutin::window::Window::set_min_inner_size`] for details.
    #[inline]
    pub fn with_logical_min_inner_size<P: Pixel>(mut self, width: P, height: P) -> Self {
        self.window_builder.window.min_inner_size = Some(LogicalSize::new(width, height).into());
        self
    }

    /// Sets a maximum dimension size for the window.
    ///
    /// See [`glutin::window::Window::set_max_inner_size`] for details.
    #[inline]
    pub fn with_max_inner_size<P: Pixel>(mut self, width: P, height: P) -> Self {
        self.window_builder.window.max_inner_size = Some(PhysicalSize::new(width, height).into());
        self
    }

    /// Sets a maximum dimension size for the window.
    ///
    /// See [`glutin::window::Window::set_max_inner_size`] for details.
    #[inline]
    pub fn with_logical_max_inner_size<P: Pixel>(mut self, width: P, height: P) -> Self {
        self.window_builder.window.max_inner_size = Some(LogicalSize::new(width, height).into());
        self
    }

    /// Sets whether the window is resizable or not.
    ///
    /// See [`glutin::window::Window::set_resizable`] for details.
    #[inline]
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.window_builder.window.resizable = resizable;
        self
    }

    /// Requests a specific title for the window.
    ///
    /// See [`glutin::window::Window::set_title`] for details.
    #[inline]
    pub fn with_title<T: Into<String>>(mut self, title: T) -> Self {
        self.window_builder.window.title = title.into();
        self
    }

    /// Sets the window fullscreen state.
    ///
    /// See [`glutin::window::Window::set_fullscreen`] for details.
    #[inline]
    pub fn with_fullscreen(mut self, fullscreen: Option<Fullscreen>) -> Self {
        self.window_builder.window.fullscreen = fullscreen;
        self
    }

    /// Requests maximized mode.
    ///
    /// See [`glutin::window::Window::set_maximized`] for details.
    #[inline]
    pub fn with_maximized(mut self, maximized: bool) -> Self {
        self.window_builder.window.maximized = maximized;
        self
    }

    /// Sets whether the window will be initially hidden or visible.
    ///
    /// See [`glutin::window::Window::set_visible`] for details.
    #[inline]
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.window_builder.window.visible = visible;
        self
    }

    /// Sets whether the background of the window should be transparent.
    #[inline]
    pub fn with_transparent(mut self, transparent: bool) -> Self {
        self.window_builder.window.transparent = transparent;
        self
    }

    /// Sets whether the window should have a border, a title bar, etc.
    ///
    /// See [`glutin::window::Window::set_decorations`] for details.
    #[inline]
    pub fn with_decorations(mut self, decorations: bool) -> Self {
        self.window_builder.window.decorations = decorations;
        self
    }

    /// Sets whether or not the window will always be on top of other windows.
    ///
    /// See [`glutin::window::Window::set_always_on_top`] for details.
    #[inline]
    pub fn with_always_on_top(mut self, always_on_top: bool) -> Self {
        self.window_builder.window.always_on_top = always_on_top;
        self
    }

    /// Sets the window icon.
    ///
    /// See [`glutin::window::Window::set_window_icon`] for details.
    #[inline]
    pub fn with_window_icon(mut self, window_icon: Option<Icon>) -> Self {
        self.window_builder.window.window_icon = window_icon;
        self
    }

    /// Sets how the backend should choose the OpenGL API and version.
    #[inline]
    pub fn with_gl(mut self, request: GlRequest) -> Self {
        self.context_builder.gl_attr.version = request;
        self
    }

    /// Sets the desired OpenGL [`glutin::Context`] profile.
    #[inline]
    pub fn with_gl_profile(mut self, profile: GlProfile) -> Self {
        self.context_builder.gl_attr.profile = Some(profile);
        self
    }

    /// Sets the *debug* flag for the OpenGL [`glutin::Context`].
    ///
    /// The default value for this flag is `cfg!(debug_assertions)`, which means
    /// that it's enabled when you run `cargo build` and disabled when you run
    /// `cargo build --release`.
    #[inline]
    pub fn with_gl_debug_flag(mut self, flag: bool) -> Self {
        self.context_builder.gl_attr.debug = flag;
        self
    }

    /// Sets the robustness of the OpenGL [`glutin::Context`]. See the docs of
    /// [`glutin::Robustness`].
    #[inline]
    pub fn with_gl_robustness(mut self, robustness: Robustness) -> Self {
        self.context_builder.gl_attr.robustness = robustness;
        self
    }

    /// Requests that the window has vsync enabled.
    ///
    /// By default, vsync is not enabled.
    #[inline]
    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.context_builder.gl_attr.vsync = vsync;
        self
    }

    /// Share the display lists with the given [`glutin::Context`].
    #[inline]
    pub fn with_shared_lists(mut self, other: &'a Context<NotCurrent>) -> Self {
        self.context_builder.gl_attr = self.context_builder.gl_attr.map_sharing(|_| other);
        self
    }

    /// Sets the multisampling level to request. A value of `0` indicates that
    /// multisampling must not be enabled.
    ///
    /// # Panic
    ///
    /// Will panic if `samples` is not a power of two.
    #[inline]
    pub fn with_multisampling(mut self, samples: u16) -> Self {
        self.context_builder.pf_reqs.multisampling = match samples {
            0 => None,
            _ => {
                assert!(samples.is_power_of_two());
                Some(samples)
            }
        };
        self
    }

    /// Sets the number of bits in the depth buffer.
    #[inline]
    pub fn with_depth_buffer(mut self, bits: u8) -> Self {
        self.context_builder.pf_reqs.depth_bits = Some(bits);
        self
    }

    /// Sets the number of bits in the stencil buffer.
    #[inline]
    pub fn with_stencil_buffer(mut self, bits: u8) -> Self {
        self.context_builder.pf_reqs.stencil_bits = Some(bits);
        self
    }

    /// Sets the number of bits in the color buffer.
    #[inline]
    pub fn with_pixel_format(mut self, color_bits: u8, alpha_bits: u8) -> Self {
        self.context_builder.pf_reqs.color_bits = Some(color_bits);
        self.context_builder.pf_reqs.alpha_bits = Some(alpha_bits);
        self
    }

    /// Request the backend to be stereoscopic.
    #[inline]
    pub fn with_stereoscopy(mut self) -> Self {
        self.context_builder.pf_reqs.stereoscopy = true;
        self
    }

    /// Sets whether sRGB should be enabled on the window.
    ///
    /// The default value is `true`.
    #[inline]
    pub fn with_srgb(mut self, enabled: bool) -> Self {
        self.context_builder.pf_reqs.srgb = enabled;
        self
    }

    /// Sets whether double buffering should be enabled.
    ///
    /// The default value is `Buffering::DoNotCare`.
    ///
    /// ## Platform-specific
    ///
    /// This option will be taken into account on the following platforms:
    ///
    ///   * MacOS
    ///   * Unix operating systems using GLX with X
    ///   * Windows using WGL
    #[inline]
    pub fn with_double_buffer<T: Into<Buffering>>(mut self, buffering: T) -> Self {
        self.context_builder.pf_reqs.double_buffer = buffering.into().into();
        self
    }

    /// Sets whether hardware acceleration is required.
    ///
    /// The default value is `Acceleration::Hardware`
    ///
    /// ## Platform-specific
    ///
    /// This option will be taken into account on the following platforms:
    ///
    ///   * MacOS
    ///   * Unix operating systems using EGL with either X or Wayland
    ///   * Windows using EGL or WGL
    ///   * Android using EGL
    #[inline]
    pub fn with_hardware_acceleration<T: Into<Acceleration>>(mut self, acceleration: T) -> Self {
        self.context_builder.pf_reqs.hardware_accelerated = acceleration.into().into();
        self
    }

    #[inline]
    pub fn with_window_builder(mut self, window_builder: WindowBuilder) -> Self {
        self.window_builder = window_builder;
        self
    }

    #[inline]
    pub fn with_context_builder(mut self, context_builder: ContextBuilder<'a, NotCurrent>) -> Self {
        self.context_builder = context_builder;
        self
    }

    #[inline]
    pub fn with_background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    #[inline]
    pub fn with_exit_by_escape(mut self, exit: bool) -> Self {
        self.exit_by_escape = exit;
        self
    }

    #[inline]
    pub fn with_font<N: Into<Cow<'a, str>>, P: Into<Cow<'a, Path>>>(mut self, name: N, path: P) -> Self {
        self.font = Some(Font {
            name: name.into(),
            path: path.into(),
        });
        self
    }

    #[inline]
    pub fn renderer(&self) -> &R {
        &self.renderer
    }

    #[inline]
    pub fn renderer_mut(&mut self) -> &mut R {
        &mut self.renderer
    }

    #[inline]
    pub fn run(self, comp: Comp) -> Result<(), AppError<R::Error>> {
        self.run_with_prerender(comp, |_, _, _| AppState::Continue)
    }

    pub fn run_with_prerender(
        self, mut comp: Comp,
        mut redraw_hook: impl FnMut(&mut Comp, &WindowedContext<PossiblyCurrent>, &mut R) -> AppState + 'static,
    ) -> Result<(), AppError<R::Error>> {
        let App {
            window_builder,
            context_builder,
            mut renderer,
            background_color,
            exit_by_escape,
            font,
        } = self;

        let event_loop = EventLoop::new();
        let context = context_builder.build_windowed(window_builder, &event_loop)?;
        let context = unsafe { context.make_current().map_err(|(_, err)| err)? };

        unsafe {
            gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);
            let color = background_color.as_arr();
            gl::ClearColor(color[0], color[1], color[2], color[3]);
        }

        let size = context.window().inner_size();
        renderer.set_dimensions(size.width, size.height, context.window().scale_factor());
        renderer.init(self.background_color).map_err(AppError::RendererError)?;
        if let Some(Font { name, path }) = font {
            renderer.load_font(name, path).map_err(AppError::RendererError)?;
        }

        let mut mouse_controller = MouseController::new();
        let keyboard_controller = KeyboardController::new();
        let mut last_time = Instant::now();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::LoopDestroyed => (),
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        context.resize(size);
                        comp.send_system_msg(SystemMessage::WindowResized {
                            width: size.width,
                            height: size.height,
                        });
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::ReceivedCharacter(ch) => {
                        keyboard_controller.input_char(&mut comp, ch);
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } if exit_by_escape => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        let KeyboardInput {
                            scancode,
                            state,
                            virtual_keycode,
                            ..
                        } = input;
                        if let ElementState::Pressed = state {
                            keyboard_controller
                                .pressed_comp(&mut comp, convert_keyboard_event(scancode, virtual_keycode));
                        } else {
                            keyboard_controller
                                .released_comp(&mut comp, convert_keyboard_event(scancode, virtual_keycode));
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        mouse_controller.update_pos(position.x as Real, position.y as Real);
                    }
                    WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        button,
                        ..
                    } => {
                        mouse_controller.pressed_comp(&mut comp, convert_mouse_button(button));
                    }
                    WindowEvent::MouseWheel {
                        delta: MouseScrollDelta::LineDelta(x, y),
                        ..
                    } => {
                        mouse_controller.mouse_scroll(&mut comp, (x, y));
                    }
                    _ => (),
                },
                Event::MainEventsCleared => {
                    context.window().request_redraw();
                }
                Event::RedrawRequested(_) => {
                    let size = context.window().inner_size();
                    unsafe {
                        gl::Viewport(0, 0, size.width as i32, size.height as i32);
                        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
                    }

                    if let AppState::Exit = redraw_hook(&mut comp, &context, &mut renderer) {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    let elapsed = last_time.elapsed();
                    last_time = Instant::now();
                    comp.send_system_msg(SystemMessage::Draw(elapsed));
                    if comp.update_view().is_some() {
                        renderer.set_dimensions(size.width, size.height, context.window().scale_factor());
                        if renderer.render(&mut comp).expect("Renderer error") {
                            context.swap_buffers().expect("Swap buffers fail");
                        }
                    } else {
                        thread::sleep(Duration::from_millis(10));
                    }
                }
                _ => (),
            }
        })
    }
}

fn convert_keyboard_event(scancode: u32, keycode: Option<VirtualKeyCode>) -> controller::KeyboardEvent {
    let keycode = keycode.map(|code| match code {
        VirtualKeyCode::Key1 => controller::VirtualKeyCode::Key1,
        VirtualKeyCode::Key2 => controller::VirtualKeyCode::Key2,
        VirtualKeyCode::Key3 => controller::VirtualKeyCode::Key3,
        VirtualKeyCode::Key4 => controller::VirtualKeyCode::Key4,
        VirtualKeyCode::Key5 => controller::VirtualKeyCode::Key5,
        VirtualKeyCode::Key6 => controller::VirtualKeyCode::Key6,
        VirtualKeyCode::Key7 => controller::VirtualKeyCode::Key7,
        VirtualKeyCode::Key8 => controller::VirtualKeyCode::Key8,
        VirtualKeyCode::Key9 => controller::VirtualKeyCode::Key9,
        VirtualKeyCode::Key0 => controller::VirtualKeyCode::Key0,
        VirtualKeyCode::A => controller::VirtualKeyCode::A,
        VirtualKeyCode::B => controller::VirtualKeyCode::B,
        VirtualKeyCode::C => controller::VirtualKeyCode::C,
        VirtualKeyCode::D => controller::VirtualKeyCode::D,
        VirtualKeyCode::E => controller::VirtualKeyCode::E,
        VirtualKeyCode::F => controller::VirtualKeyCode::F,
        VirtualKeyCode::G => controller::VirtualKeyCode::G,
        VirtualKeyCode::H => controller::VirtualKeyCode::H,
        VirtualKeyCode::I => controller::VirtualKeyCode::I,
        VirtualKeyCode::J => controller::VirtualKeyCode::J,
        VirtualKeyCode::K => controller::VirtualKeyCode::K,
        VirtualKeyCode::L => controller::VirtualKeyCode::L,
        VirtualKeyCode::M => controller::VirtualKeyCode::M,
        VirtualKeyCode::N => controller::VirtualKeyCode::N,
        VirtualKeyCode::O => controller::VirtualKeyCode::O,
        VirtualKeyCode::P => controller::VirtualKeyCode::P,
        VirtualKeyCode::Q => controller::VirtualKeyCode::Q,
        VirtualKeyCode::R => controller::VirtualKeyCode::R,
        VirtualKeyCode::S => controller::VirtualKeyCode::S,
        VirtualKeyCode::T => controller::VirtualKeyCode::T,
        VirtualKeyCode::U => controller::VirtualKeyCode::U,
        VirtualKeyCode::V => controller::VirtualKeyCode::V,
        VirtualKeyCode::W => controller::VirtualKeyCode::W,
        VirtualKeyCode::X => controller::VirtualKeyCode::X,
        VirtualKeyCode::Y => controller::VirtualKeyCode::Y,
        VirtualKeyCode::Z => controller::VirtualKeyCode::Z,
        VirtualKeyCode::Escape => controller::VirtualKeyCode::Escape,
        VirtualKeyCode::F1 => controller::VirtualKeyCode::F1,
        VirtualKeyCode::F2 => controller::VirtualKeyCode::F2,
        VirtualKeyCode::F3 => controller::VirtualKeyCode::F3,
        VirtualKeyCode::F4 => controller::VirtualKeyCode::F4,
        VirtualKeyCode::F5 => controller::VirtualKeyCode::F5,
        VirtualKeyCode::F6 => controller::VirtualKeyCode::F6,
        VirtualKeyCode::F7 => controller::VirtualKeyCode::F7,
        VirtualKeyCode::F8 => controller::VirtualKeyCode::F8,
        VirtualKeyCode::F9 => controller::VirtualKeyCode::F9,
        VirtualKeyCode::F10 => controller::VirtualKeyCode::F10,
        VirtualKeyCode::F11 => controller::VirtualKeyCode::F11,
        VirtualKeyCode::F12 => controller::VirtualKeyCode::F12,
        VirtualKeyCode::F13 => controller::VirtualKeyCode::F13,
        VirtualKeyCode::F14 => controller::VirtualKeyCode::F14,
        VirtualKeyCode::F15 => controller::VirtualKeyCode::F15,
        VirtualKeyCode::F16 => controller::VirtualKeyCode::F16,
        VirtualKeyCode::F17 => controller::VirtualKeyCode::F17,
        VirtualKeyCode::F18 => controller::VirtualKeyCode::F18,
        VirtualKeyCode::F19 => controller::VirtualKeyCode::F19,
        VirtualKeyCode::F20 => controller::VirtualKeyCode::F20,
        VirtualKeyCode::F21 => controller::VirtualKeyCode::F21,
        VirtualKeyCode::F22 => controller::VirtualKeyCode::F22,
        VirtualKeyCode::F23 => controller::VirtualKeyCode::F23,
        VirtualKeyCode::F24 => controller::VirtualKeyCode::F24,
        VirtualKeyCode::Snapshot => controller::VirtualKeyCode::Snapshot,
        VirtualKeyCode::Scroll => controller::VirtualKeyCode::Scroll,
        VirtualKeyCode::Pause => controller::VirtualKeyCode::Pause,
        VirtualKeyCode::Insert => controller::VirtualKeyCode::Insert,
        VirtualKeyCode::Home => controller::VirtualKeyCode::Home,
        VirtualKeyCode::Delete => controller::VirtualKeyCode::Delete,
        VirtualKeyCode::End => controller::VirtualKeyCode::End,
        VirtualKeyCode::PageDown => controller::VirtualKeyCode::PageDown,
        VirtualKeyCode::PageUp => controller::VirtualKeyCode::PageUp,
        VirtualKeyCode::Left => controller::VirtualKeyCode::Left,
        VirtualKeyCode::Up => controller::VirtualKeyCode::Up,
        VirtualKeyCode::Right => controller::VirtualKeyCode::Right,
        VirtualKeyCode::Down => controller::VirtualKeyCode::Down,
        VirtualKeyCode::Back => controller::VirtualKeyCode::Backspace,
        VirtualKeyCode::Return => controller::VirtualKeyCode::Enter,
        VirtualKeyCode::Space => controller::VirtualKeyCode::Space,
        VirtualKeyCode::Compose => controller::VirtualKeyCode::Compose,
        VirtualKeyCode::Caret => controller::VirtualKeyCode::Caret,
        VirtualKeyCode::Numlock => controller::VirtualKeyCode::Numlock,
        VirtualKeyCode::Numpad0 => controller::VirtualKeyCode::Numpad0,
        VirtualKeyCode::Numpad1 => controller::VirtualKeyCode::Numpad1,
        VirtualKeyCode::Numpad2 => controller::VirtualKeyCode::Numpad2,
        VirtualKeyCode::Numpad3 => controller::VirtualKeyCode::Numpad3,
        VirtualKeyCode::Numpad4 => controller::VirtualKeyCode::Numpad4,
        VirtualKeyCode::Numpad5 => controller::VirtualKeyCode::Numpad5,
        VirtualKeyCode::Numpad6 => controller::VirtualKeyCode::Numpad6,
        VirtualKeyCode::Numpad7 => controller::VirtualKeyCode::Numpad7,
        VirtualKeyCode::Numpad8 => controller::VirtualKeyCode::Numpad8,
        VirtualKeyCode::Numpad9 => controller::VirtualKeyCode::Numpad9,
        VirtualKeyCode::NumpadAdd => controller::VirtualKeyCode::NumpadAdd,
        VirtualKeyCode::NumpadDivide => controller::VirtualKeyCode::NumpadDivide,
        VirtualKeyCode::NumpadDecimal => controller::VirtualKeyCode::NumpadDecimal,
        VirtualKeyCode::NumpadComma => controller::VirtualKeyCode::NumpadComma,
        VirtualKeyCode::NumpadEnter => controller::VirtualKeyCode::NumpadEnter,
        VirtualKeyCode::NumpadEquals => controller::VirtualKeyCode::NumpadEquals,
        VirtualKeyCode::NumpadMultiply => controller::VirtualKeyCode::NumpadMultiply,
        VirtualKeyCode::NumpadSubtract => controller::VirtualKeyCode::NumpadSubtract,
        VirtualKeyCode::AbntC1 => controller::VirtualKeyCode::AbntC1,
        VirtualKeyCode::AbntC2 => controller::VirtualKeyCode::AbntC2,
        VirtualKeyCode::Apostrophe => controller::VirtualKeyCode::Apostrophe,
        VirtualKeyCode::Apps => controller::VirtualKeyCode::Apps,
        VirtualKeyCode::Asterisk => controller::VirtualKeyCode::Asterisk,
        VirtualKeyCode::At => controller::VirtualKeyCode::At,
        VirtualKeyCode::Ax => controller::VirtualKeyCode::Ax,
        VirtualKeyCode::Backslash => controller::VirtualKeyCode::Backslash,
        VirtualKeyCode::Calculator => controller::VirtualKeyCode::Calculator,
        VirtualKeyCode::Capital => controller::VirtualKeyCode::Capital,
        VirtualKeyCode::Colon => controller::VirtualKeyCode::Colon,
        VirtualKeyCode::Comma => controller::VirtualKeyCode::Comma,
        VirtualKeyCode::Convert => controller::VirtualKeyCode::Convert,
        VirtualKeyCode::Equals => controller::VirtualKeyCode::Equals,
        VirtualKeyCode::Grave => controller::VirtualKeyCode::Grave,
        VirtualKeyCode::Kana => controller::VirtualKeyCode::Kana,
        VirtualKeyCode::Kanji => controller::VirtualKeyCode::Kanji,
        VirtualKeyCode::LAlt => controller::VirtualKeyCode::LAlt,
        VirtualKeyCode::LBracket => controller::VirtualKeyCode::LBracket,
        VirtualKeyCode::LControl => controller::VirtualKeyCode::LControl,
        VirtualKeyCode::LShift => controller::VirtualKeyCode::LShift,
        VirtualKeyCode::LWin => controller::VirtualKeyCode::LWin,
        VirtualKeyCode::Mail => controller::VirtualKeyCode::Mail,
        VirtualKeyCode::MediaSelect => controller::VirtualKeyCode::MediaSelect,
        VirtualKeyCode::MediaStop => controller::VirtualKeyCode::MediaStop,
        VirtualKeyCode::Minus => controller::VirtualKeyCode::Minus,
        VirtualKeyCode::Mute => controller::VirtualKeyCode::Mute,
        VirtualKeyCode::MyComputer => controller::VirtualKeyCode::MyComputer,
        VirtualKeyCode::NavigateForward => controller::VirtualKeyCode::NavigateForward,
        VirtualKeyCode::NavigateBackward => controller::VirtualKeyCode::NavigateBackward,
        VirtualKeyCode::NextTrack => controller::VirtualKeyCode::NextTrack,
        VirtualKeyCode::NoConvert => controller::VirtualKeyCode::NoConvert,
        VirtualKeyCode::OEM102 => controller::VirtualKeyCode::OEM102,
        VirtualKeyCode::Period => controller::VirtualKeyCode::Period,
        VirtualKeyCode::PlayPause => controller::VirtualKeyCode::PlayPause,
        VirtualKeyCode::Plus => controller::VirtualKeyCode::Plus,
        VirtualKeyCode::Power => controller::VirtualKeyCode::Power,
        VirtualKeyCode::PrevTrack => controller::VirtualKeyCode::PrevTrack,
        VirtualKeyCode::RAlt => controller::VirtualKeyCode::RAlt,
        VirtualKeyCode::RBracket => controller::VirtualKeyCode::RBracket,
        VirtualKeyCode::RControl => controller::VirtualKeyCode::RControl,
        VirtualKeyCode::RShift => controller::VirtualKeyCode::RShift,
        VirtualKeyCode::RWin => controller::VirtualKeyCode::RWin,
        VirtualKeyCode::Semicolon => controller::VirtualKeyCode::Semicolon,
        VirtualKeyCode::Slash => controller::VirtualKeyCode::Slash,
        VirtualKeyCode::Sleep => controller::VirtualKeyCode::Sleep,
        VirtualKeyCode::Stop => controller::VirtualKeyCode::Stop,
        VirtualKeyCode::Sysrq => controller::VirtualKeyCode::Sysrq,
        VirtualKeyCode::Tab => controller::VirtualKeyCode::Tab,
        VirtualKeyCode::Underline => controller::VirtualKeyCode::Underline,
        VirtualKeyCode::Unlabeled => controller::VirtualKeyCode::Unlabeled,
        VirtualKeyCode::VolumeDown => controller::VirtualKeyCode::VolumeDown,
        VirtualKeyCode::VolumeUp => controller::VirtualKeyCode::VolumeUp,
        VirtualKeyCode::Wake => controller::VirtualKeyCode::Wake,
        VirtualKeyCode::WebBack => controller::VirtualKeyCode::WebBack,
        VirtualKeyCode::WebFavorites => controller::VirtualKeyCode::WebFavorites,
        VirtualKeyCode::WebForward => controller::VirtualKeyCode::WebForward,
        VirtualKeyCode::WebHome => controller::VirtualKeyCode::WebHome,
        VirtualKeyCode::WebRefresh => controller::VirtualKeyCode::WebRefresh,
        VirtualKeyCode::WebSearch => controller::VirtualKeyCode::WebSearch,
        VirtualKeyCode::WebStop => controller::VirtualKeyCode::WebStop,
        VirtualKeyCode::Yen => controller::VirtualKeyCode::Yen,
        VirtualKeyCode::Copy => controller::VirtualKeyCode::Copy,
        VirtualKeyCode::Paste => controller::VirtualKeyCode::Paste,
        VirtualKeyCode::Cut => controller::VirtualKeyCode::Cut,
    });
    controller::KeyboardEvent { scancode, keycode }
}

fn convert_mouse_button(button: MouseButton) -> controller::MouseButton {
    match button {
        MouseButton::Left => controller::MouseButton::Left,
        MouseButton::Right => controller::MouseButton::Right,
        MouseButton::Middle => controller::MouseButton::Middle,
        MouseButton::Other(code) => controller::MouseButton::Other(code),
    }
}
