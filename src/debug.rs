use ash::{ext, vk};

pub struct DebugDong {
  pub loader: ext::debug_utils::Instance,
  pub messenger: vk::DebugUtilsMessengerEXT,
}

impl DebugDong {
  pub fn info() -> vk::DebugUtilsMessengerCreateInfoEXT<'static> {
    vk::DebugUtilsMessengerCreateInfoEXT::default()
      .message_severity(
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
          | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
          | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
          | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
      )
      .message_type(
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
          | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
          | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
      )
      .pfn_user_callback(Some(vulkan_debug_utils_callback))
  }

  pub fn init(
    entry: &ash::Entry,
    instance: &ash::Instance,
    debug_create_info: &vk::DebugUtilsMessengerCreateInfoEXT,
  ) -> Result<Self, Box<dyn std::error::Error>> {
    let loader = ext::debug_utils::Instance::new(entry, instance);
    let messenger = unsafe { loader.create_debug_utils_messenger(debug_create_info, None) }?;
    Ok(Self { loader, messenger })
  }
}

impl Drop for DebugDong {
  fn drop(&mut self) {
    unsafe {
      self
        .loader
        .destroy_debug_utils_messenger(self.messenger, None);
    }
  }
}

unsafe extern "system" fn vulkan_debug_utils_callback(
  message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
  message_type: vk::DebugUtilsMessageTypeFlagsEXT,
  p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
  _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
  let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
  let severity = format!("{:?}", message_severity).to_lowercase();
  let ty = format!("{:?}", message_type).to_lowercase();
  if severity == "info" {
    let msg=message.to_str().expect("An error occurred in Vulkan debug utils callback. What kind of not-String are you handing me?");
    if msg.contains("DEBUG-PRINTF") {
      let msg = msg
        .to_string()
        .replace("Validation Information: [ UNASSIGNED-DEBUG-PRINTF ]", "");
      println!("[Debug][printf] {:?}", msg);
    }
  } else {
    println!("[Debug][{}][{}] {:?}", severity, ty, message);
  }
  vk::FALSE
}
