//! A Surfman platform crate for the Luminance graphics API
//!
//! This crate is useful in situtions where you do not have control over the window creation, and
//! you need to be able to create a Luminance surface after the window and event loop have already
//! been created.
//!
//! This crate currently supports creating a Luminance surface from a winit window, but could also
//! be easily extended to allow you to create surfaces from a [raw window handle][rwh]. Open an
//! issue if you have that use case!
//!
//! [rwh]: https://docs.rs/raw-window-handle/0.3.3/raw_window_handle/

use euclid::Size2D;
use luminance::{
    context::GraphicsContext,
    framebuffer::{Framebuffer, FramebufferError},
    texture::Dim2,
};
pub use luminance_glow::ShaderVersion;
use luminance_glow::{Context as GlowContext, Glow, StateQueryError};
use surfman::{
    Connection, Context, ContextAttributeFlags, ContextAttributes, Device, GLVersion,
    SurfaceAccess, SurfaceType,
};
use winit::window::Window;

surfman::declare_surfman!();

#[derive(thiserror::Error, Debug)]
pub enum SurfmanError {
    #[error("Surface error: {0}")]
    SurfaceError(String),
    #[error("GL error: {0}")]
    GlError(#[from] StateQueryError),
    #[error("Framebuffer error: {0}")]
    FramebufferError(#[from] FramebufferError),
}

pub struct SurfmanSurface {
    backend: Glow,
    device: Device,
    context: Context,
}

unsafe impl GraphicsContext for SurfmanSurface {
    type Backend = Glow;

    fn backend(&mut self) -> &mut Self::Backend {
        &mut self.backend
    }
}

impl SurfmanSurface {
    /// Create a surface from a winit window
    ///
    /// > ⚠️ **Warning:** Because the surfman surface does not have access to the window event loop
    /// > you will need to manualy call [`set_size`] on the surface when the window is resized.
    pub fn from_winit_window(
        window: &Window,
        shader_version: ShaderVersion,
    ) -> Result<Self, SurfmanError> {
        // Create a connection to the graphics provider from our winit window
        let conn = Connection::from_winit_window(&window).map_err(surface_err)?;
        // Create a native widget to attach the visible render surface to
        let native_widget = conn
            .create_native_widget_from_winit_window(&window)
            .map_err(surface_err)?;
        // Create a hardware adapter that we can used to create graphics devices from
        let adapter = conn.create_hardware_adapter().map_err(surface_err)?;
        // Create a graphics device using our hardware adapter
        let mut device = conn.create_device(&adapter).map_err(surface_err)?;

        // Define the attributes for our OpenGL context
        let context_attributes = ContextAttributes {
            version: GLVersion::new(3, 3),
            flags: ContextAttributeFlags::ALPHA
                | ContextAttributeFlags::DEPTH
                | ContextAttributeFlags::STENCIL,
        };

        // Create a context descriptor based on our defined context attributes
        let context_descriptor = device
            .create_context_descriptor(&context_attributes)
            .map_err(surface_err)?;
        // Define the surface type for our graphics surface ( a surface based on a native widget, i.e. not an offscreen surface )
        let surface_type = SurfaceType::Widget { native_widget };
        // Create an OpenGL context
        let mut context = device
            .create_context(&context_descriptor, None)
            .map_err(surface_err)?;

        // Create a surface that can be accessed only from the GPU
        let surface = device
            .create_surface(&context, SurfaceAccess::GPUCPU, surface_type)
            .map_err(surface_err)?;

        // Bind our surface to our create GL context
        device
            .bind_surface_to_context(&mut context, surface)
            .map_err(|(e, _)| surface_err(e))?;
        // Make our context the current context
        device.make_context_current(&context).map_err(surface_err)?;

        // Get a pointer to the OpenGL functions
        let gl = unsafe {
            GlowContext::from_loader_function(
                |s| device.get_proc_address(&context, s) as *const _,
                shader_version,
            )
        };

        let backend = Glow::from_context(gl)?;

        Ok(SurfmanSurface {
            backend,
            device,
            context,
        })
    }

    /// Get the back buffer
    pub fn back_buffer(&mut self) -> Result<Framebuffer<Glow, Dim2, (), ()>, SurfmanError> {
        let surface = self
            .device
            .unbind_surface_from_context(&mut self.context)
            .map_err(surface_err)?
            .unwrap();

        let surface_info = self.device.surface_info(&surface);
        let width = surface_info.size.width as u32;
        let height = surface_info.size.height as u32;

        self.device
            .bind_surface_to_context(&mut self.context, surface)
            .map_err(|(e, _)| surface_err(e))?;

        Ok(Framebuffer::back_buffer(self, [width, height])?)
    }

    /// Swap the front and back buffers
    pub fn swap_buffers(&mut self) -> Result<(), SurfmanError> {
        let mut surface = self
            .device
            .unbind_surface_from_context(&mut self.context)
            .map_err(surface_err)?
            .unwrap();
        self.device
            .present_surface(&self.context, &mut surface)
            .map_err(surface_err)?;
        self.device
            .bind_surface_to_context(&mut self.context, surface)
            .map_err(|(e, _)| surface_err(e))?;

        Ok(())
    }

    /// Set the size of the surface
    pub fn set_size(&mut self, size: [u32; 2]) -> Result<(), SurfmanError> {
        let mut surface = self
            .device
            .unbind_surface_from_context(&mut self.context)
            .map_err(surface_err)?
            .unwrap();
        self.device
            .resize_surface(
                &mut self.context,
                &mut surface,
                Size2D::new(size[0] as i32, size[1] as i32),
            )
            .map_err(surface_err)?;
        self.device
            .bind_surface_to_context(&mut self.context, surface)
            .map_err(|(e, _)| surface_err(e))?;

        Ok(())
    }
}

impl Drop for SurfmanSurface {
    fn drop(&mut self) {
        self.device
            .destroy_context(&mut self.context)
            .unwrap_or_else(|e| eprintln!("Error destroying surfman context: {:?}", e));
    }
}

// Util to format map a surfman error to this crate's [`SurfmanError`]
fn surface_err(e: surfman::Error) -> SurfmanError {
    SurfmanError::SurfaceError(format!("{:?}", e))
}
