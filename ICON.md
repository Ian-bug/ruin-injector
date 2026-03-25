# Ruin DLL Injector - Icon Usage Guide

## 📌 Icon Placement

Place your icon file (.ico format) in the project root directory:

```
rust-injector/
├── icon.ico          ← Put it here!
├── extension_icon512.png  ← Source PNG icon
├── extension_icon1024.png ← Source PNG icon (high-res)
├── src/
├── Cargo.toml
├── build.rs
└── README.md
```

## 📐 Icon Requirements

### Recommended Format:
- **Format**: .ico (Windows standard icon format)
- **Sizes**: Recommended to include multiple sizes
  - 16x16 (Small icon in taskbar)
  - 32x32 (Desktop icon)
  - 48x48 (Medium icon)
  - 256x256 (High-definition display)

### How to Create .ico Files:

#### Method 1: Online Conversion
Use online tools to convert PNG to ICO:
- https://icoconvert.com/
- https://www.icoconverter.com/
- https://convertico.com/

#### Method 2: Using Tools
- **GIMP**: Export as ICO format
- **Photoshop**: Use ICO plugin
- **IcoFX**: Dedicated ICO editor

#### Method 3: Using Code
If you have a PNG icon, you can convert it using ImageMagick:
```bash
magick icon.png -define icon:auto-resize=256,128,96,64,48,32,16 icon.ico
```

**Current Project Example**:
```bash
# Convert the 1024px PNG icon to ICO format
magick extension_icon1024.png -define icon:auto-resize=256,128,96,64,48,32,16 icon.ico
```

## 🚀 Usage Steps

1. **Prepare Icon**: Convert your icon to .ico format
2. **Place File**: Put `icon.ico` in the `rust-injector` folder root
3. **Rebuild**: Run `cargo build --release`
4. **Check Result**: Compilation output will show "Icon set: icon.ico"

## 📋 Notes

- Filename must be `icon.ico` (lowercase)
- .ico format must include 256x256 size for best results
- Compilation will automatically embed icon into executable file

## 🎨 Example Icon Design

Recommended icon design:
- Use clean syringe or needle icon
- Theme color: Red/Orange (indicating injection)
- Background: Transparent or white
- Style: Flat or subtle gradient

## ❓ Common Questions

**Q: Can I use PNG?**
A: No, build.rs currently only supports .ico format. You need to convert first.

**Q: What to do after changing icon?**
A: Just re-run `cargo build --release`, icon will be embedded automatically.

**Q: Icon not changing?**
A: Make sure:
1. Filename is `icon.ico` (not Icon.ico or ICON.ico)
2. File is in correct directory (`rust-injector/` folder)
3. Clean and rebuild: `cargo clean && cargo build --release`
