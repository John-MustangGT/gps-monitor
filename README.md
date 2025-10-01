# GPS Monitor - Modern egui GPS Monitoring Tool

A cross-platform GPS monitoring application with a modern graphical interface built with egui. Monitor GPS data from serial ports, gpsd daemon, or Windows Location Services.

![GPS Monitor Screenshot](screenshot.png)

## Features

- 🌍 **Multi-Source Support**
  - Serial GPS devices (NMEA)
  - gpsd daemon
  - Windows Location Services
  
- 🎨 **Modern GUI Interface**
  - Real-time satellite sky plot
  - Sortable satellite table with signal quality indicators
  - Position, movement, and signal quality displays
  - NMEA sentence history viewer
  
- ⚙️ **Integrated Settings**
  - Easy source switching with GUI settings dialog
  - Persistent configuration (Registry on Windows, JSON on Linux)
  - Hot reconnect without app restart
  
- 🛰️ **Multi-GNSS Support**
  - GPS (USA)
  - GLONASS (Russia)
  - Galileo (EU)
  - BeiDou (China)
  - QZSS (Japan)
  - SBAS

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/gps-monitor
cd gps-monitor

# Build and run
cargo build --release --features gui
cargo run --release --features gui
```

### System Requirements

**Linux:**
- X11 or Wayland display server
- Required packages: `libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev`

**Windows:**
- Windows 10 or later
- No additional dependencies

## Usage

### Starting the Application

```bash
# Run with default configuration
gps-monitor

# Or explicitly with features
cargo run --release --features gui
```

### First Launch

On first launch, GPS Monitor will use platform-specific defaults:
- **Windows**: Windows Location Services
- **Linux**: gpsd at localhost:2947

### Configuring GPS Source

Click the **⚙ Settings** button in the top menu bar to open the settings dialog:

1. **Serial Port**
   - Select for direct GPS device connection
   - Configure port (e.g., COM3, /dev/ttyUSB0) and baud rate

2. **gpsd**
   - Select for gpsd daemon connection
   - Configure host and port (default: localhost:2947)

3. **Windows Location** (Windows only)
   - Select for Windows Location Services
   - Configure accuracy and update interval

Changes are automatically saved and loaded on next launch.

### UI Controls

**Top Menu Bar:**
- **Status Indicator**: Shows connection state (green = connected, yellow = waiting, red = disconnected)
- **▶ Connect / ⏸ Disconnect**: Control GPS connection
- **🔄 Restart**: Restart GPS connection
- **⚙ Settings**: Open settings dialog
- **❌ Exit**: Close application

**Main Display:**
- **Left Panel**: Position, movement, and signal quality
- **Right Top**: Satellite sky plot (polar view)
- **Right Bottom**: Sortable satellite table (click headers to sort)
- **Bottom Panel**: NMEA sentences / raw data stream

### Satellite Table

Click any column header to sort:
- **Constellation**: Sort by GNSS system
- **PRN**: Sort by satellite ID
- **Used**: Sort by whether used in fix
- **SNR (dB)**: Sort by signal strength
- **Quality**: Sort by signal quality (Excellent → Very Poor)
- **Elevation**: Sort by elevation angle
- **Azimuth**: Sort by azimuth angle

## Configuration Storage

### Windows
Settings are stored in the Windows Registry:
```
HKEY_CURRENT_USER\Software\GpsMonitor
```

### Linux/Unix
Settings are stored in a JSON file:
```
~/.config/gps-monitor/config.json
```

## Building

### Standard Build (GUI enabled)
```bash
cargo build --release --features gui
```

### Development Build
```bash
cargo build --features gui
```

### Make Commands
```bash
make release-gui      # Build release with GUI
make build-gui        # Build debug with GUI
make setup-gui        # Install GUI system dependencies (Linux)
make display-check    # Check display environment
```

## Dependencies

### Runtime Dependencies

**Linux:**
- X11 or Wayland
- OpenGL drivers
- For serial: Read/write permissions on serial devices (`/dev/ttyUSB*`, etc.)
- For gpsd: Running gpsd daemon

**Windows:**
- For Windows Location: Location services enabled in Windows Settings

### Development Dependencies

See `Cargo.toml` for complete list. Key dependencies:
- `eframe` / `egui`: GUI framework
- `tokio`: Async runtime
- `tokio-serial`: Serial port communication
- `serde` / `serde_json`: Configuration serialization
- `chrono`: Date/time handling
- Platform-specific: `windows` crate, `winreg` (Windows only)

## Architecture

```
src/
├── main.rs              # Application entry point
├── lib.rs               # Library exports
├── config.rs            # Configuration management
├── error.rs             # Error types
├── monitor.rs           # GPS monitoring coordination
├── gps/
│   ├── data.rs         # GPS data structures
│   ├── nmea.rs         # NMEA parser
│   ├── gpsd.rs         # gpsd client
│   └── windows.rs      # Windows Location API
└── display/
    └── gui/
        ├── mod.rs      # GUI module exports
        ├── app.rs      # Main application & eframe::App
        ├── panels.rs   # Data display panels
        ├── satellites.rs # Satellite table
        ├── skyplot.rs  # Sky plot visualization
        └── settings.rs # Settings dialog
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details

## Acknowledgments

- Built with [egui](https://github.com/emilk/egui) - immediate mode GUI framework
- GPS parsing inspired by various NMEA libraries
- Icon set from [Lucide](https://lucide.dev/)

## Troubleshooting

### Linux: No display found
Ensure `DISPLAY` or `WAYLAND_DISPLAY` environment variable is set:
```bash
echo $DISPLAY
make display-check
```

### Linux: Serial port permission denied
Add user to `dialout` group:
```bash
sudo usermod -a -G dialout $USER
# Log out and back in for changes to take effect
```

### Windows: Location services not working
1. Open Windows Settings
2. Go to Privacy & Security → Location
3. Enable "Location services"
4. Enable location access for apps

### gpsd: Connection refused
Ensure gpsd is running:
```bash
sudo systemctl status gpsd
# Or start it:
sudo systemctl start gpsd
```

## Support

For issues, questions, or suggestions, please open an issue on GitHub.
