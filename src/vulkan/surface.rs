use anyhow::Error;
use ash::{khr, vk};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub(crate) struct Surface {
  surface_loader: khr::surface::Instance,
  surface: vk::SurfaceKHR,
}

impl Surface {
  pub(crate) fn init(
    entry: &ash::Entry,
    instance: &ash::Instance,
    window: &winit::window::Window,
  ) -> Result<Self, Error> {
    let display_handle = window.display_handle().unwrap().as_raw();
    let window_handle = window.window_handle().unwrap().as_raw();
    let surface =
      unsafe { ash_window::create_surface(entry, instance, display_handle, window_handle, None) }?;
    let surface_loader = khr::surface::Instance::new(entry, instance);

    Ok(Self {
      surface_loader,
      surface,
    })
  }

  pub(crate) fn get_capabilities(
    &self,
    physical_device: vk::PhysicalDevice,
  ) -> Result<vk::SurfaceCapabilitiesKHR, vk::Result> {
    unsafe {
      self
        .surface_loader
        .get_physical_device_surface_capabilities(physical_device, self.surface)
    }
  }

  fn get_present_modes(
    &self,
    physical_device: vk::PhysicalDevice,
  ) -> Result<Vec<vk::PresentModeKHR>, vk::Result> {
    unsafe {
      self
        .surface_loader
        .get_physical_device_surface_present_modes(physical_device, self.surface)
    }
  }

  pub(crate) fn get_formats(
    &self,
    physical_device: vk::PhysicalDevice,
  ) -> Result<Vec<vk::SurfaceFormatKHR>, vk::Result> {
    unsafe {
      self
        .surface_loader
        .get_physical_device_surface_formats(physical_device, self.surface)
    }
  }

  pub(crate) fn get_support(
    &self,
    physical_device: vk::PhysicalDevice,
    queue_family_index: u32,
  ) -> Result<bool, vk::Result> {
    unsafe {
      self.surface_loader.get_physical_device_surface_support(
        physical_device,
        queue_family_index,
        self.surface,
      )
    }
  }

  pub(crate) fn get_surface(&self) -> vk::SurfaceKHR {
    self.surface
  }

  pub(crate) fn destroy(&self) {
    unsafe {
      self.surface_loader.destroy_surface(self.surface, None);
    }
  }
}
