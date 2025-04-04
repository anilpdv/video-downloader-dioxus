# Resources Directory

This directory contains pre-compiled binaries of `yt-dlp` for various platforms. These binaries are embedded in the application at compile time and extracted at runtime to ensure that the application does not depend on users having `yt-dlp` installed on their system.

## Platform-specific binaries

The following binaries are included:

- `yt-dlp-windows-x64.exe`: Windows 64-bit
- `yt-dlp-windows-x86.exe`: Windows 32-bit
- `yt-dlp-macos-x64`: macOS Intel (64-bit)
- `yt-dlp-macos-arm64`: macOS Apple Silicon (ARM64)
- `yt-dlp-linux-x64`: Linux 64-bit
- `yt-dlp-linux-arm64`: Linux ARM64

## Source and Licensing

These binaries are sourced from the official [yt-dlp GitHub repository](https://github.com/yt-dlp/yt-dlp/releases) and are subject to their licensing terms. yt-dlp is licensed under the Unlicense license.

## Update Process

To update these binaries, download the latest versions from the [yt-dlp releases page](https://github.com/yt-dlp/yt-dlp/releases) and replace the files in this directory. Then rebuild the application to include the updated versions.

```bash
# Update Windows binaries
curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe -o resources/yt-dlp-windows-x64.exe
cp resources/yt-dlp-windows-x64.exe resources/yt-dlp-windows-x86.exe

# Update macOS binaries
curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos -o resources/yt-dlp-macos-x64
cp resources/yt-dlp-macos-x64 resources/yt-dlp-macos-arm64

# Update Linux binaries
curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o resources/yt-dlp-linux-x64
cp resources/yt-dlp-linux-x64 resources/yt-dlp-linux-arm64 