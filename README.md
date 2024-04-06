# BMS Kneeboard Server
#### For OpenKneeboard 1.7+

## About

BMS-Kneeboard-Server is a tool that transforms Falcon BMS's `briefing.txt` into a HTML server that can be used in OpenKneeboard's **Web Dashboard** tab, or, as a side effect of how it works, it can also be displayed on an external device such as a tablet.

## Prerequisites

Ensure `1. Briefing Output to File` is **checked**, and `3. HTML Briefings` is **not** checked in the config.

![Config][config-manager]

## Usage

![Tray Icon][trayicon]

BMS Kneeboard Server (BKS) lives in your system tray.

Rightclicking the icon gives the following options:

- `http://127.0.0.1:7878/`<sup>1</sup> Clicking this option will open the page in your default browser.
- `Help` will open this page.
- `Exit` Shuts down the server.

> <sup>1</sup> Default address, it changes depending on given command line options.

## Sections

You can build up a custom kneeboard from multiple sections:

Append to the URL `http://127.0.0.1:7878/` any order and combination of the following:
- `MO` - Mission Overview
- `SR` - Sitrep
- `PR` - Pilot Roster
- `PE` - Package Elements
- `TA` - Threat Analysis
- `SP` - Steerpoints
- `CL` - Comm Ladder
- `IF` - IFF
- `OR` - Ordnance
- `WT` - Weather
- `SU` - Support
- `RO` - Rules of Engagement
- `EP` - Emergency Procedures

The default value is `PESPCL`, showing: `P`ackage `E`lements, `S`teer `P`oints, and `C`omm `L`adder: `http://127.0.0.1:7878/PESPCL`.

In OpenKneeboard, go to the settings (cogwheel in the bottom left corner), navigate to **Tabs**, click on the button that says **+ Add a tab**, and choose **Web Dashboard**. The following dialog will ask you for a dashboard address, this is where you link OpenKneeboard with the server by entering the URL. You can create multiple Web Dashboard tabs with different selections of sections.

![Preview][preview]

## Command-line arguments

```
Usage: bms-kneeboard-server.exe [OPTIONS] [BRIEFING_DIR]

Arguments:
  [BRIEFING_DIR]  Override directory containing briefing.txt, disabled autodetect

Options:
  -l, --listen <LISTEN>  Webserver listen address:port [default: 127.0.0.1:7878]
  -h, --help             Print help
  -V, --version          Print version
```

### Hidden feature (ssh, don't tell anyone)
If you append `#width=2048` or some other number to the url `http://127.0.0.1:7878/#width=...` you can override the rendering width of the page. Default is `1024`

[find-local-ip]: https://support.microsoft.com/en-us/windows/find-your-ip-address-in-windows-f21a9bbc-c582-55cd-35e0-73431160a1b9
[config-manager]: assets/config-manager.png
[preview]: assets/preview.png
[trayicon]: assets/trayicon.png
