use anyhow::Error;
use ash::{ext, vk::{self, ExtendsInstanceCreateInfo}};

pub(super) const VALIDATION_LAYER: &std::ffi::CStr = unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") };

pub(crate) struct Debugger {
  debug_utils: DebugUtils,
}

impl Debugger {
  pub(crate) fn init(
    entry: &ash::Entry,
    instance: &ash::Instance,
    debugger_info: DebuggerInfo,
  ) -> Result<Self, Error> {
    let debug_utils = DebugUtils::init(entry, instance, &debugger_info.debug_utils)?;

    Ok(Self { debug_utils })
  }

  pub(crate) fn info() -> DebuggerInfo {
    DebuggerInfo {
      debug_utils: vk::DebugUtilsMessengerCreateInfoEXT::default()
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
        .pfn_user_callback(Some(vulkan_debug_utils_callback)),
    }
  }

  pub(crate) fn destroy(&mut self) {
    self.debug_utils.destroy();
  }
}

pub(crate) struct DebugUtils {
  loader: ext::debug_utils::Instance,
  messenger: vk::DebugUtilsMessengerEXT,
}

impl DebugUtils {
  pub(crate) fn init(
    entry: &ash::Entry,
    instance: &ash::Instance,
    debug_create_info: &vk::DebugUtilsMessengerCreateInfoEXT,
  ) -> Result<Self, Error> {
    let loader = ext::debug_utils::Instance::new(entry, instance);
    let messenger = unsafe { loader.create_debug_utils_messenger(debug_create_info, None) }?;
    Ok(Self { loader, messenger })
  }

  pub(crate) fn destroy(&mut self) {
    unsafe {
      self
        .loader
        .destroy_debug_utils_messenger(self.messenger, None);
    }
  }
}

#[derive(Debug)]
pub(crate) struct DebuggerInfo {
  debug_utils: vk::DebugUtilsMessengerCreateInfoEXT<'static>,
}

impl DebuggerInfo {
  pub(crate) fn instance_next(&mut self) -> Vec<Box<dyn ExtendsInstanceCreateInfo>> {
    vec![Box::new(self.debug_utils)]
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
