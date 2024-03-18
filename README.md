# Falcon BMS Kneeboard Server
#### For OpenKneeboard 1.7+

## About

BMS-Kneeboard-Server is a tool that transforms Falcon BMS's `briefing.txt` into a HTML server that can be used in OpenKneeboard's new **Web Dashboard** tab, or, as a side effect of how it works, it can also be displayed on an external device such as a tablet.

## Usage

By default, when you run `bms-kneeboard-server.exe` it'll autodetect your Falcon BMS installation, the server will wait for you to start one of the games and automatically pick up which one you've started. Alternatively, you can set the path of the briefings directory to something arbitrary as the first argument. `bms-kneeboard-server.exe "C:\Some Dir\Where\Briefings\Are"`

The full output from the server's console window will look something like this:

```
Multiple BMS installations detected
Waiting for Falcon BMS
Listening on http://127.0.0.1:7878
Watching [D:\Games\Falcon BMS 4.37\User\Briefings\briefing.txt] for changes
```

The important part to take from this is the `listening on` line; this is required for OpenKneeboard. By default this is the address `http://127.0.0.1:7878`, however, if you override the listen address using `--listen 0.0.0.0:7878` (to open the page on an external device), this line will show all zeros, which is _not_ a valid address. In this case, you need to find out what your computer's [local ip address is][find-local-ip] replace `0.0.0.0` with that instead.

In OpenKneeboard, go to the settings (cogwheel in the bottom left corner), navigate to **Tabs**, click on the button that says **+ Add a tab**, and choose **Web Dashboard**. The following dialog will ask you for a dashboard address, this is where you link OpenKneeboard with the server by entering the previously mentioned address.

> If **Web Dashboard** is not available in the Add a tab list, your OpenKneeboard version is most likely out of date. Update OpenKneeboard by pressing the questionmark in the bottom left and clicking the **Check for Updates** button.

To update the kneeboard, all you have to do is hit the **Print** button on the briefing screen in Falcon BMS. If the kneeboard does not update, ensure that `Briefing Output to File` is enabled and `HTML Briefings` is disabled in the [Config][config-manager].

If all is well, you should see something like this:

![Preview][preview]

## Command-line arguments

```
Usage: bms-kneeboard-server.exe [OPTIONS] [BRIEFING_DIR]

Arguments:
  [BRIEFING_DIR]  Override Falcon BMS Briefing Path

Options:
  -l, --listen <LISTEN>  Webserver listen address:port [default: 127.0.0.1:7878]
  -h, --help             Print help
  -V, --version          Print version
```

[find-local-ip]: https://support.microsoft.com/en-us/windows/find-your-ip-address-in-windows-f21a9bbc-c582-55cd-35e0-73431160a1b9
[config-manager]: assets/config-manager.png
[preview]: assets/preview.png
