{ pkgs, ... }:

{
  packages = with pkgs; [
    pkg-config
    vulkan-tools
  ];

  env.LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
    with pkgs;
    [
      libGL
      libxkbcommon
      wayland
      vulkan-loader
    ]
  );
}
