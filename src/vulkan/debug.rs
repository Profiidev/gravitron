use anyhow::Error;
use ash::{
  ext,
  vk::{self, ExtendsInstanceCreateInfo},
};

use super::VulkanConfig;

const VALIDATION_LAYER: &std::ffi::CStr =
  unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") };

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

  pub(crate) fn init_info(vulkan_config: &mut VulkanConfig) -> DebuggerInfo {
    let is_info_level = vulkan_config.debug_log_level.contains(vk::DebugUtilsMessageSeverityFlagsEXT::INFO);

    let mut debugger_info = DebuggerInfo {
      debug_utils: vk::DebugUtilsMessengerCreateInfoEXT::default()
        .message_severity(vulkan_config.debug_log_level)
        .message_type(
          vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(vulkan_debug_utils_callback))
        .user_data(if is_info_level { 1 as *mut _ } else { std::ptr::null_mut() }),
    };

    vulkan_config.layers.push(VALIDATION_LAYER);

    let validation_ext = vk::ValidationFeaturesEXT::default()
      .enabled_validation_features(&[vk::ValidationFeatureEnableEXT::DEBUG_PRINTF]);
    vulkan_config.instance_next.push(Box::new(validation_ext));

    vulkan_config
      .instance_extensions
      .push(ext::debug_report::NAME);
    vulkan_config
      .instance_extensions
      .push(ext::debug_utils::NAME);

    vulkan_config
      .instance_next
      .append(&mut debugger_info.instance_next());

    debugger_info
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
  p_user_data: *mut std::ffi::c_void,
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
    } else if !p_user_data.is_null() {
      println!("[Debug][{}][{}] {:?}", severity, ty, message);
    }
  } else {
    println!("[Debug][{}][{}] {:?}", severity, ty, message);
  }

  vk::FALSE
}
