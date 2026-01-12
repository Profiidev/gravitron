# Changelog

All notable changes to this project will be documented in this file.

## [0.4.2] - 2026-01-12



## [0.4.1] - 2025-05-01

### 🚀 Features

- Added global transform


## [0.4.0] - 2025-01-21

### 🚀 Features

- Faster hashmap
- Added inline in ecs
- Added added and changed to components
- Save comp removed tick
- Added filter to query
- Inital hierarchy implementation
- Added simpler way to propergate top down through hierarchy
- Added option for changing system execution type

### 🐛 Bug Fixes

- Archetype res invalid
- Fixed some oversights in rework

### 🚜 Refactor

- Moved id to struct
- Moved modelid to struct
- Changed query structure
- Removed combined ecs struct and made scheduler pub
- Renamed type_ to r#type

### 📚 Documentation

- Fixed readme

### ⚡ Performance

- Added some more benches
- Removed into_query overhead

### 🎨 Styling

- Fixed format

### 🧪 Testing

- Fixed benches
- Added query filter tests

### ⚙️ Miscellaneous Tasks

- Moved some dependencies to workspace
- Fixed windows tests


## [0.3.0] - 2024-10-29

### 🚀 Features

- Added possibiliy for settings relative execution order
- Generic as stage identifier

### 🐛 Bug Fixes

- Ecs storage edges correct id
- Set componentid type correctly
- TypeId randomly different
- ResMut wrong acces type
- Suboptimal system parallelization
- Wrong access using unsafe world cell

### 🚜 Refactor

- Switched to typeid for components
- Only compile trace logs if using debug feature

### ⚙️ Miscellaneous Tasks

- Fixed typo


## [0.2.0] - 2024-10-02

### 🚀 Features

- [**breaking**] Ecs builder pattern for systems and resources
- *(ecs)* Create_entity in commands no returns the id
- Added capability for parallel system execution
- Added logging ([#32](https://github.com/Profiidev/gravitron/pull/32))
- Added ecs to gravitron
- Added ability to set resources after building the ecs and retriving them
- Made UnsageWorldCell publicly available

### 🐛 Bug Fixes

- [**breaking**] Moved create entity to ecs builder

### 🚜 Refactor

- Removed builder pattern from ecs
- Moved systemparams

### 🧪 Testing

- Added tests for meta

### ⚙️ Miscellaneous Tasks

- Updated READMEs


## [0.1.2] - 2024-09-13

### 🧪 Testing

- Ecs now has tests ([#21](https://github.com/Profiidev/gravitron/pull/21))


## [0.1.1] - 2024-09-13

### ⚙️ Miscellaneous Tasks

- Release ([#2](https://github.com/Profiidev/gravitron/pull/2))
- Release ([#17](https://github.com/Profiidev/gravitron/pull/17))


