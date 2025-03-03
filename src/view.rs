//! The zombiesplit user interface.
pub mod config;
pub mod error;
mod event;
pub mod gfx;

use crate::presenter;
use std::cell::RefCell;

pub use config::Config;
pub use error::{Error, Result};

/// A top-level view, owning the various UI resources.
pub struct View {
    sdl: sdl2::Sdl,
    screen: RefCell<sdl2::render::Canvas<sdl2::video::Window>>,
    textures: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    cfg: Config,
}

impl View {
    /// Creates a new view, opening a window in the process.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the SDL subsystems the UI manager requires
    /// fail to initialise.
    pub fn new(cfg: Config) -> Result<Self> {
        let sdl = sdl2::init().map_err(Error::Init)?;
        let video = sdl.video().map_err(Error::Init)?;
        let window = gfx::make_window(&video, cfg.window)?;
        let screen = window.into_canvas().build().map_err(Error::SdlInteger)?;
        let textures = screen.texture_creator();
        Ok(Self {
            sdl,
            screen: RefCell::new(screen),
            textures,
            cfg,
        })
    }

    /// Spawns a [Core] handling UI services.
    ///
    /// # Errors
    ///
    /// Returns an error if SDL can't spawn an event pump.
    pub fn spawn(&self, presenter: presenter::Presenter) -> Result<Instance> {
        let font_manager =
            gfx::font::Manager::new(&self.textures, &self.cfg.fonts, &self.cfg.colours);
        let renderer = gfx::render::Window::new(
            self.screen.borrow_mut(),
            self.cfg.window,
            font_manager,
            &self.cfg.colours,
        )?;
        let gfx = gfx::Core::new(renderer, self.cfg.window);

        let events = self.sdl.event_pump().map_err(Error::Init)?;

        Ok(Instance {
            events,
            gfx,
            presenter,
        })
    }
}

/// An instance of the view for a particular presenter.
pub struct Instance<'a> {
    events: sdl2::EventPump,
    gfx: gfx::Core<'a>,
    presenter: presenter::Presenter,
}

impl<'a> Instance<'a> {
    /// Runs the UI loop.
    ///
    /// # Errors
    ///
    /// Returns an error if SDL fails to perform an action.
    pub fn run(&mut self) -> error::Result<()> {
        // TODO(@MattWindsor91): pass in something other than Game.

        self.gfx.redraw(&self.presenter)?;

        while self.presenter.is_running() {
            for e in self.events.poll_iter() {
                if let Some(x) = event::from_sdl(&e) {
                    self.presenter.handle_event(&x)
                }
            }
            self.gfx.redraw(&self.presenter)?;
        }

        Ok(())
    }
}
