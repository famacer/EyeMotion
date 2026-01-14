# Changelog

All notable changes to this project will be documented in this file.

## [1.1.0] - 2026-01-15

### 🚀 Features & Gameplay
- **Stage 3 Adjustment**: Vertical speed is now set to 0.7x of horizontal speed for a more balanced tracking challenge.
- **Stage 2/4 Optimization**: Reduced overall global speed to improve smooth pursuit training efficacy.
- **Stage 2 Redesign**: Motion pattern changed from pure vertical to "Near Vertical" (random 5-15° angle) to prevent training adaptation.

### 💅 UI/UX Improvements
- **Visual Clarity**: Implemented pixel-perfect alignment for buttons to eliminate edge blurring (毛边).
- **Aesthetics**: Refined button stroke rendering with "Inside" alignment and a secondary anti-aliasing pass.
- **Visual Comfort**: Adjusted stroke transparency to 30% for better visual integration.

### ⚡ Performance
- **High Refresh Rate**: Verified and optimized engine for 240Hz-540Hz displays using `requestAnimationFrame`.
- **Size Optimization**: Reduced executable size from ~11MB to ~4.6MB via Rust LTO and symbol stripping.

### 📝 Documentation
- Added English README (`README.md`).
- Renamed Chinese manual to `README_ZH.md`.
- Added MIT License.

---

## [1.0.0] - 2026-01-11
- Initial release.
- Basic linear and circular tracking modes.
- Sound effects and background music.
