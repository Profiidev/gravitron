# Changelog

All notable changes to this project will be documented in this file.

## [0.3.0] - 2024-10-18

### 🚀 Features

- First working meshrenderer
- Added delta time
- Added systemstages
- Switched to global gpu managment
- Added Descriptor updating
- Added BufferBlockSize for easier control
- Indirect indexed drawing
- Added buffermemory resize to memorymanager
- Added simple buffer for smaller memory amount
- Distinct types for buffers
- Image sampler in descriptor
- Added uvs to models
- Added support for multiple images in one descriptor
- Added texture loading for fragment shaders
- Made textures working in default shader
- Made ecs macros work in every crate
- Added ability for including images as bytes

### 🐛 Bug Fixes

- Corrected roation of transfrom
- Removed remaining code errors for buffer rework
- Smarter instancedata sizing and worng instancedata sizing
- Loaded correct index data
- Memory manager not destroying fences
- Incorrect shader mem creation
- Insufficent memory allocation for large amount of new instances
- Wrong copy of modified instance data
- Wrong instance index in draw command
- Wrong drawcmd copy
- Wrong isntance id after mem resize
- Wrong access using unsafe world cell
- Exported shader macros

### 🚜 Refactor

- Removed old render code
- Only compile trace logs if using debug feature
- Moved render pass to new file
- Moved managed buffer to seperate file
- Made vertex shader hardcoded
- Hardcoded default descriptor
- Reduced camera data to one buffer
- Unified advanced and simple buffer types into one
- Unified buffer and image memory types
- Combined cmd buffer and fence to transfer
- Moved shaders to assets

### ⚙️ Miscellaneous Tasks

- Moved test files
- Fixed readme and cargo toml


## [0.2.0] - 2024-10-02

### 🚀 Features

- [**breaking**] Ecs builder pattern for systems and resources
- Added capability for parallel system execution
- Added logging ([#32](https://github.com/Profiidev/gravitron/pull/32))
- Added ecs to gravitron
- Clear color render
- First ecs integration

### 🐛 Bug Fixes

- Made debugger import feature conditional
- Vulkan wait for idle device before destroy

### 🚜 Refactor

- Removed unused stuff
- Debug is now a feature

### 🧪 Testing

- Added tests for macos ([#24](https://github.com/Profiidev/gravitron/pull/24))

### ⚙️ Miscellaneous Tasks

- Removed unneccessary dependency
- Excluded lock file from crate
- Updated READMEs
- Removed cargo dist and switched to release-plz github releases


## [0.1.2] - 2024-09-13

### 🐛 Bug Fixes

- Ninja not in windows runner ([#19](https://github.com/Profiidev/gravitron/pull/19))
- No macos imports ([#23](https://github.com/Profiidev/gravitron/pull/23))

### 🧪 Testing

- Ecs now has tests ([#21](https://github.com/Profiidev/gravitron/pull/21))
- Added text results as comment to pr ([#22](https://github.com/Profiidev/gravitron/pull/22))


## [0.1.1] - 2024-09-13

### ⚙️ Miscellaneous Tasks

- Release ([#2](https://github.com/Profiidev/gravitron/pull/2))
- Release ([#17](https://github.com/Profiidev/gravitron/pull/17))


